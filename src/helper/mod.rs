// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022-2023 hinto-janai
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// This file represents the "helper" thread, which is the full separate thread
// that runs alongside the main [App] GUI thread. It exists for the entire duration
// of Gupax so that things can be handled without locking up the GUI thread.
//
// This thread is a continual 1 second loop, collecting available jobs on the
// way down and (if possible) asynchronously executing them at the very end.
//
// The main GUI thread will interface with this thread by mutating the Arc<Mutex>'s
// found here, e.g: User clicks [Stop P2Pool] -> Arc<Mutex<ProcessSignal> is set
// indicating to this thread during its loop: "I should stop P2Pool!", e.g:
//
//     if lock!(p2pool).signal == ProcessSignal::Stop {
//         stop_p2pool(),
//     }
//
// This also includes all things related to handling the child processes (P2Pool/XMRig):
// piping their stdout/stderr/stdin, accessing their APIs (HTTP + disk files), etc.

//---------------------------------------------------------------------------------------------------- Import
use crate::helper::{
    p2pool::{ImgP2pool, PubP2poolApi},
    xmrig::{ImgXmrig, PubXmrigApi},
};
use crate::{constants::*, disk::gupax_p2pool_api::GupaxP2poolApi, human::*, macros::*};
use log::*;
use std::path::Path;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::*,
};
pub mod p2pool;
pub mod xmrig;

//---------------------------------------------------------------------------------------------------- Constants
// The max amount of bytes of process output we are willing to
// hold in memory before it's too much and we need to reset.
const MAX_GUI_OUTPUT_BYTES: usize = 500_000;
// Just a little leeway so a reset will go off before the [String] allocates more memory.
const GUI_OUTPUT_LEEWAY: usize = MAX_GUI_OUTPUT_BYTES - 1000;

// Some constants for generating hashrate/difficulty.
const MONERO_BLOCK_TIME_IN_SECONDS: u64 = 120;
const P2POOL_BLOCK_TIME_IN_SECONDS: u64 = 10;

//---------------------------------------------------------------------------------------------------- [Helper] Struct
// A meta struct holding all the data that gets processed in this thread
pub struct Helper {
    pub instant: Instant,                             // Gupax start as an [Instant]
    pub uptime: HumanTime,                            // Gupax uptime formatting for humans
    pub pub_sys: Arc<Mutex<Sys>>, // The public API for [sysinfo] that the [Status] tab reads from
    pub p2pool: Arc<Mutex<Process>>, // P2Pool process state
    pub xmrig: Arc<Mutex<Process>>, // XMRig process state
    pub gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for GUI thread)
    pub gui_api_xmrig: Arc<Mutex<PubXmrigApi>>, // XMRig API state (for GUI thread)
    pub img_p2pool: Arc<Mutex<ImgP2pool>>, // A static "image" of the data P2Pool started with
    pub img_xmrig: Arc<Mutex<ImgXmrig>>, // A static "image" of the data XMRig started with
    pub_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for Helper/P2Pool thread)
    pub_api_xmrig: Arc<Mutex<PubXmrigApi>>, // XMRig API state (for Helper/XMRig thread)
    pub gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>, //
}

// The communication between the data here and the GUI thread goes as follows:
// [GUI] <---> [Helper] <---> [Watchdog] <---> [Private Data only available here]
//
// Both [GUI] and [Helper] own their separate [Pub*Api] structs.
// Since P2Pool & XMRig will be updating their information out of sync,
// it's the helpers job to lock everything, and move the watchdog [Pub*Api]s
// on a 1-second interval into the [GUI]'s [Pub*Api] struct, atomically.

//----------------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Sys {
    pub gupax_uptime: String,
    pub gupax_cpu_usage: String,
    pub gupax_memory_used_mb: String,
    pub system_cpu_model: String,
    pub system_memory: String,
    pub system_cpu_usage: String,
}

impl Sys {
    pub fn new() -> Self {
        Self {
            gupax_uptime: "0 seconds".to_string(),
            gupax_cpu_usage: "???%".to_string(),
            gupax_memory_used_mb: "??? megabytes".to_string(),
            system_cpu_usage: "???%".to_string(),
            system_memory: "???GB / ???GB".to_string(),
            system_cpu_model: "???".to_string(),
        }
    }
}
impl Default for Sys {
    fn default() -> Self {
        Self::new()
    }
}

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
#[derive(Debug)]
pub struct Process {
    pub name: ProcessName,     // P2Pool or XMRig?
    pub state: ProcessState,   // The state of the process (alive, dead, etc)
    pub signal: ProcessSignal, // Did the user click [Start/Stop/Restart]?
    // STDIN Problem:
    //     - User can input many many commands in 1 second
    //     - The process loop only processes every 1 second
    //     - If there is only 1 [String] holding the user input,
    //       the user could overwrite their last input before
    //       the loop even has a chance to process their last command
    // STDIN Solution:
    //     - When the user inputs something, push it to a [Vec]
    //     - In the process loop, loop over every [Vec] element and
    //       send each one individually to the process stdin
    //
    pub input: Vec<String>,

    // The below are the handles to the actual child process.
    // [Simple] has no STDIN, but [Advanced] does. A PTY (pseudo-terminal) is
    // required for P2Pool/XMRig to open their STDIN pipe.
    //	child: Option<Arc<Mutex<Box<dyn portable_pty::Child + Send + std::marker::Sync>>>>, // STDOUT/STDERR is combined automatically thanks to this PTY, nice
    //	stdin: Option<Box<dyn portable_pty::MasterPty + Send>>, // A handle to the process's MasterPTY/STDIN

    // This is the process's private output [String], used by both [Simple] and [Advanced].
    // "parse" contains the output that will be parsed, then tossed out. "pub" will be written to
    // the same as parse, but it will be [swap()]'d by the "helper" thread into the GUIs [String].
    // The "helper" thread synchronizes this swap so that the data in here is moved there
    // roughly once a second. GUI thread never touches this.
    output_parse: Arc<Mutex<String>>,
    output_pub: Arc<Mutex<String>>,

    // Start time of process.
    start: std::time::Instant,
}

//---------------------------------------------------------------------------------------------------- [Process] Impl
impl Process {
    pub fn new(name: ProcessName, _args: String, _path: PathBuf) -> Self {
        Self {
            name,
            state: ProcessState::Dead,
            signal: ProcessSignal::None,
            start: Instant::now(),
            //			stdin: Option::None,
            //			child: Option::None,
            output_parse: arc_mut!(String::with_capacity(500)),
            output_pub: arc_mut!(String::with_capacity(500)),
            input: vec![String::new()],
        }
    }

    // Borrow a [&str], return an owned split collection
    #[inline]
    pub fn parse_args(args: &str) -> Vec<String> {
        args.split_whitespace().map(|s| s.to_owned()).collect()
    }

    #[inline]
    // Convenience functions
    pub fn is_alive(&self) -> bool {
        self.state == ProcessState::Alive
            || self.state == ProcessState::Middle
            || self.state == ProcessState::Syncing
            || self.state == ProcessState::NotMining
    }

    #[inline]
    pub fn is_waiting(&self) -> bool {
        self.state == ProcessState::Middle || self.state == ProcessState::Waiting
    }

    #[inline]
    pub fn is_syncing(&self) -> bool {
        self.state == ProcessState::Syncing
    }

    #[inline]
    pub fn is_not_mining(&self) -> bool {
        self.state == ProcessState::NotMining
    }
}

//---------------------------------------------------------------------------------------------------- [Process*] Enum
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessState {
    Alive,   // Process is online, GREEN!
    Dead,    // Process is dead, BLACK!
    Failed,  // Process is dead AND exited with a bad code, RED!
    Middle,  // Process is in the middle of something ([re]starting/stopping), YELLOW!
    Waiting, // Process was successfully killed by a restart, and is ready to be started again, YELLOW!

    // Only for P2Pool, ORANGE.
    Syncing,

    // Only for XMRig, ORANGE.
    NotMining,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self::Dead
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessSignal {
    None,
    Start,
    Stop,
    Restart,
}

impl Default for ProcessSignal {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessName {
    P2pool,
    Xmrig,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::fmt::Display for ProcessSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::fmt::Display for ProcessName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProcessName::P2pool => write!(f, "P2Pool"),
            ProcessName::Xmrig => write!(f, "XMRig"),
        }
    }
}

//---------------------------------------------------------------------------------------------------- [Helper]
impl Helper {
    //---------------------------------------------------------------------------------------------------- General Functions
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instant: std::time::Instant,
        pub_sys: Arc<Mutex<Sys>>,
        p2pool: Arc<Mutex<Process>>,
        xmrig: Arc<Mutex<Process>>,
        gui_api_p2pool: Arc<Mutex<PubP2poolApi>>,
        gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,
        img_p2pool: Arc<Mutex<ImgP2pool>>,
        img_xmrig: Arc<Mutex<ImgXmrig>>,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
    ) -> Self {
        Self {
            instant,
            pub_sys,
            uptime: HumanTime::into_human(instant.elapsed()),
            pub_api_p2pool: arc_mut!(PubP2poolApi::new()),
            pub_api_xmrig: arc_mut!(PubXmrigApi::new()),
            // These are created when initializing [App], since it needs a handle to it as well
            p2pool,
            xmrig,
            gui_api_p2pool,
            gui_api_xmrig,
            img_p2pool,
            img_xmrig,
            gupax_p2pool_api,
        }
    }

    // Reset output if larger than max bytes.
    // This will also append a message showing it was reset.
    fn check_reset_gui_output(output: &mut String, name: ProcessName) {
        let len = output.len();
        if len > GUI_OUTPUT_LEEWAY {
            info!(
                "{} Watchdog | Output is nearing {} bytes, resetting!",
                name, MAX_GUI_OUTPUT_BYTES
            );
            let text = format!("{}\n{} GUI log is exceeding the maximum: {} bytes!\nI've reset the logs for you!\n{}\n\n\n\n", HORI_CONSOLE, name, MAX_GUI_OUTPUT_BYTES, HORI_CONSOLE);
            output.clear();
            output.push_str(&text);
            debug!("{} Watchdog | Resetting GUI output ... OK", name);
        } else {
            debug!(
                "{} Watchdog | GUI output reset not needed! Current byte length ... {}",
                name, len
            );
        }
    }

    // Read P2Pool/XMRig's API file to a [String].
    fn path_to_string(
        path: &Path,
        name: ProcessName,
    ) -> std::result::Result<String, std::io::Error> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("{} API | [{}] read error: {}", name, path.display(), e);
                Err(e)
            }
        }
    }
    //---------------------------------------------------------------------------------------------------- The "helper"
    #[inline(always)] // called once
    fn update_pub_sys_from_sysinfo(
        sysinfo: &sysinfo::System,
        pub_sys: &mut Sys,
        pid: &sysinfo::Pid,
        helper: &Helper,
        max_threads: usize,
    ) {
        let gupax_uptime = helper.uptime.to_string();
        let cpu = &sysinfo.cpus()[0];
        let gupax_cpu_usage = format!(
            "{:.2}%",
            sysinfo.process(*pid).unwrap().cpu_usage() / (max_threads as f32)
        );
        let gupax_memory_used_mb =
            HumanNumber::from_u64(sysinfo.process(*pid).unwrap().memory() / 1_000_000);
        let gupax_memory_used_mb = format!("{} megabytes", gupax_memory_used_mb);
        let system_cpu_model = format!("{} ({}MHz)", cpu.brand(), cpu.frequency());
        let system_memory = {
            let used = (sysinfo.used_memory() as f64) / 1_000_000_000.0;
            let total = (sysinfo.total_memory() as f64) / 1_000_000_000.0;
            format!("{:.3} GB / {:.3} GB", used, total)
        };
        let system_cpu_usage = {
            let mut total: f32 = 0.0;
            for cpu in sysinfo.cpus() {
                total += cpu.cpu_usage();
            }
            format!("{:.2}%", total / (max_threads as f32))
        };
        *pub_sys = Sys {
            gupax_uptime,
            gupax_cpu_usage,
            gupax_memory_used_mb,
            system_cpu_usage,
            system_memory,
            system_cpu_model,
        };
    }

    #[cold]
    #[inline(never)]
    // The "helper" thread. Syncs data between threads here and the GUI.
    #[allow(clippy::await_holding_lock)]
    pub fn spawn_helper(
        helper: &Arc<Mutex<Self>>,
        mut sysinfo: sysinfo::System,
        pid: sysinfo::Pid,
        max_threads: usize,
    ) {
        // The ordering of these locks is _very_ important. They MUST be in sync with how the main GUI thread locks stuff
        // or a deadlock will occur given enough time. They will eventually both want to lock the [Arc<Mutex>] the other
        // thread is already locking. Yes, I figured this out the hard way, hence the vast amount of debug!() messages.
        // Example of different order (BAD!):
        //
        // GUI Main       -> locks [p2pool] first
        // Helper         -> locks [gui_api_p2pool] first
        // GUI Status Tab -> tries to lock [gui_api_p2pool] -> CAN'T
        // Helper         -> tries to lock [p2pool] -> CAN'T
        //
        // These two threads are now in a deadlock because both
        // are trying to access locks the other one already has.
        //
        // The locking order here must be in the same chronological
        // order as the main GUI thread (top to bottom).

        let helper = Arc::clone(helper);
        let lock = lock!(helper);
        let p2pool = Arc::clone(&lock.p2pool);
        let xmrig = Arc::clone(&lock.xmrig);
        let pub_sys = Arc::clone(&lock.pub_sys);
        let gui_api_p2pool = Arc::clone(&lock.gui_api_p2pool);
        let gui_api_xmrig = Arc::clone(&lock.gui_api_xmrig);
        let pub_api_p2pool = Arc::clone(&lock.pub_api_p2pool);
        let pub_api_xmrig = Arc::clone(&lock.pub_api_xmrig);
        drop(lock);

        let sysinfo_cpu = sysinfo::CpuRefreshKind::everything();
        let sysinfo_processes = sysinfo::ProcessRefreshKind::new().with_cpu();

        thread::spawn(move || {
            info!("Helper | Hello from helper thread! Entering loop where I will spend the rest of my days...");
            // Begin loop
            loop {
                // 1. Loop init timestamp
                let start = Instant::now();
                debug!("Helper | ----------- Start of loop -----------");

                // Ignore the invasive [debug!()] messages on the right side of the code.
                // The reason why they are there are so that it's extremely easy to track
                // down the culprit of an [Arc<Mutex>] deadlock. I know, they're ugly.

                // 2. Lock... EVERYTHING!
                let mut lock = lock!(helper);
                debug!("Helper | Locking (1/9) ... [helper]");
                let p2pool = lock!(p2pool);
                debug!("Helper | Locking (2/9) ... [p2pool]");
                let xmrig = lock!(xmrig);
                debug!("Helper | Locking (3/9) ... [xmrig]");
                let mut lock_pub_sys = lock!(pub_sys);
                debug!("Helper | Locking (5/9) ... [pub_sys]");
                let mut gui_api_p2pool = lock!(gui_api_p2pool);
                debug!("Helper | Locking (6/9) ... [gui_api_p2pool]");
                let mut gui_api_xmrig = lock!(gui_api_xmrig);
                debug!("Helper | Locking (7/9) ... [gui_api_xmrig]");
                let mut pub_api_p2pool = lock!(pub_api_p2pool);
                debug!("Helper | Locking (8/9) ... [pub_api_p2pool]");
                let mut pub_api_xmrig = lock!(pub_api_xmrig);
                debug!("Helper | Locking (9/9) ... [pub_api_xmrig]");
                // Calculate Gupax's uptime always.
                lock.uptime = HumanTime::into_human(lock.instant.elapsed());
                // If [P2Pool] is alive...
                if p2pool.is_alive() {
                    debug!("Helper | P2Pool is alive! Running [combine_gui_pub_api()]");
                    PubP2poolApi::combine_gui_pub_api(&mut gui_api_p2pool, &mut pub_api_p2pool);
                } else {
                    debug!("Helper | P2Pool is dead! Skipping...");
                }
                // If [XMRig] is alive...
                if xmrig.is_alive() {
                    debug!("Helper | XMRig is alive! Running [combine_gui_pub_api()]");
                    PubXmrigApi::combine_gui_pub_api(&mut gui_api_xmrig, &mut pub_api_xmrig);
                } else {
                    debug!("Helper | XMRig is dead! Skipping...");
                }

                // 2. Selectively refresh [sysinfo] for only what we need (better performance).
                sysinfo.refresh_cpu_specifics(sysinfo_cpu);
                debug!("Helper | Sysinfo refresh (1/3) ... [cpu]");
                sysinfo.refresh_processes_specifics(sysinfo_processes);
                debug!("Helper | Sysinfo refresh (2/3) ... [processes]");
                sysinfo.refresh_memory();
                debug!("Helper | Sysinfo refresh (3/3) ... [memory]");
                debug!("Helper | Sysinfo OK, running [update_pub_sys_from_sysinfo()]");
                Self::update_pub_sys_from_sysinfo(
                    &sysinfo,
                    &mut lock_pub_sys,
                    &pid,
                    &lock,
                    max_threads,
                );

                // 3. Drop... (almost) EVERYTHING... IN REVERSE!
                drop(lock_pub_sys);
                debug!("Helper | Unlocking (1/9) ... [pub_sys]");
                drop(xmrig);
                debug!("Helper | Unlocking (2/9) ... [xmrig]");
                drop(p2pool);
                debug!("Helper | Unlocking (3/9) ... [p2pool]");
                drop(pub_api_xmrig);
                debug!("Helper | Unlocking (4/9) ... [pub_api_xmrig]");
                drop(pub_api_p2pool);
                debug!("Helper | Unlocking (5/9) ... [pub_api_p2pool]");
                drop(gui_api_xmrig);
                debug!("Helper | Unlocking (6/9) ... [gui_api_xmrig]");
                drop(gui_api_p2pool);
                debug!("Helper | Unlocking (7/9) ... [gui_api_p2pool]");
                drop(lock);
                debug!("Helper | Unlocking (8/9) ... [helper]");

                // 4. Calculate if we should sleep or not.
                // If we should sleep, how long?
                let elapsed = start.elapsed().as_millis();
                if elapsed < 1000 {
                    // Casting from u128 to u64 should be safe here, because [elapsed]
                    // is less than 1000, meaning it can fit into a u64 easy.
                    let sleep = (1000 - elapsed) as u64;
                    debug!("Helper | END OF LOOP - Sleeping for [{}]ms...", sleep);
                    sleep!(sleep);
                } else {
                    debug!("Helper | END OF LOOP - Not sleeping!");
                }

                // 5. End loop
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
    use crate::helper::p2pool::{PrivP2poolLocalApi, PrivP2poolNetworkApi};

    use super::*;

    #[test]
    fn reset_gui_output() {
        let max = crate::helper::GUI_OUTPUT_LEEWAY;
        let mut string = String::with_capacity(max);
        for _ in 0..=max {
            string.push('0');
        }
        Helper::check_reset_gui_output(&mut string, ProcessName::P2pool);
        // Some text gets added, so just check for less than 500 bytes.
        assert!(string.len() < 500);
    }

    #[test]
    fn combine_gui_pub_p2pool_api() {
        use crate::helper::PubP2poolApi;
        let mut gui_api = PubP2poolApi::new();
        let mut pub_api = PubP2poolApi::new();
        pub_api.payouts = 1;
        pub_api.payouts_hour = 2.0;
        pub_api.payouts_day = 3.0;
        pub_api.payouts_month = 4.0;
        pub_api.xmr = 1.0;
        pub_api.xmr_hour = 2.0;
        pub_api.xmr_day = 3.0;
        pub_api.xmr_month = 4.0;
        println!("BEFORE - GUI_API: {:#?}\nPUB_API: {:#?}", gui_api, pub_api);
        assert_ne!(gui_api, pub_api);
        PubP2poolApi::combine_gui_pub_api(&mut gui_api, &mut pub_api);
        println!("AFTER - GUI_API: {:#?}\nPUB_API: {:#?}", gui_api, pub_api);
        assert_eq!(gui_api, pub_api);
        pub_api.xmr = 2.0;
        PubP2poolApi::combine_gui_pub_api(&mut gui_api, &mut pub_api);
        assert_eq!(gui_api, pub_api);
        assert_eq!(gui_api.xmr, 2.0);
        assert_eq!(pub_api.xmr, 2.0);
    }

    #[test]
    fn calc_payouts_and_xmr_from_output_p2pool() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			payout of 5.000000000001 XMR in block 1112
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        let public = public.lock().unwrap();
        println!("{:#?}", public);
        assert_eq!(public.payouts, 3);
        assert_eq!(public.payouts_hour, 180.0);
        assert_eq!(public.payouts_day, 4320.0);
        assert_eq!(public.payouts_month, 129600.0);
        assert_eq!(public.xmr, 15.000000000003);
        assert_eq!(public.xmr_hour, 900.00000000018);
        assert_eq!(public.xmr_day, 21600.00000000432);
        assert_eq!(public.xmr_month, 648000.0000001296);
    }

    #[test]
    fn set_p2pool_synchronized() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn p2pool_synchronized_false_positive() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));

        // The SideChain that is "SYNCHRONIZED" in this output is
        // probably not main/mini, but the sidechain started on height 1,
        // so this should _not_ trigger alive state.
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			SideChain new chain tip: next height = 1
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Syncing); // still syncing
    }

    #[test]
    fn p2pool_synchronized_double_synchronized() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));

        // The 1st SideChain that is "SYNCHRONIZED" in this output is
        // the sidechain started on height 1, but there is another one
        // which means the real main/mini is probably synced,
        // so this _should_ trigger alive state.
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			SideChain new chain tip: next height = 1
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113
			NOTICE  2021-12-27 21:42:17.2100 SideChain SYNCHRONIZED"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn update_pub_p2pool_from_local_network_pool() {
        use crate::helper::p2pool::PoolStatistics;
        use crate::helper::p2pool::PrivP2poolLocalApi;
        use crate::helper::p2pool::PrivP2poolNetworkApi;
        use crate::helper::p2pool::PrivP2poolPoolApi;
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let local = PrivP2poolLocalApi {
            hashrate_15m: 10_000,
            hashrate_1h: 20_000,
            hashrate_24h: 30_000,
            shares_found: 1000,
            average_effort: 100.000,
            current_effort: 200.000,
            connections: 1234,
        };
        let network = PrivP2poolNetworkApi {
            difficulty: 300_000_000_000,
            hash: "asdf".to_string(),
            height: 1234,
            reward: 2345,
            timestamp: 3456,
        };
        let pool = PrivP2poolPoolApi {
            pool_statistics: PoolStatistics {
                hashRate: 1_000_000, // 1 MH/s
                miners: 1_000,
            },
        };
        // Update Local
        PubP2poolApi::update_from_local(&public, local);
        let p = public.lock().unwrap();
        println!("AFTER LOCAL: {:#?}", p);
        assert_eq!(p.hashrate_15m.to_string(), "10,000");
        assert_eq!(p.hashrate_1h.to_string(), "20,000");
        assert_eq!(p.hashrate_24h.to_string(), "30,000");
        assert_eq!(p.shares_found.to_string(), "1,000");
        assert_eq!(p.average_effort.to_string(), "100.00%");
        assert_eq!(p.current_effort.to_string(), "200.00%");
        assert_eq!(p.connections.to_string(), "1,234");
        assert_eq!(p.user_p2pool_hashrate_u64, 20000);
        drop(p);
        // Update Network + Pool
        PubP2poolApi::update_from_network_pool(&public, network, pool);
        let p = public.lock().unwrap();
        println!("AFTER NETWORK+POOL: {:#?}", p);
        assert_eq!(p.monero_difficulty.to_string(), "300,000,000,000");
        assert_eq!(p.monero_hashrate.to_string(), "2.500 GH/s");
        assert_eq!(p.hash.to_string(), "asdf");
        assert_eq!(p.height.to_string(), "1,234");
        assert_eq!(p.reward.to_u64(), 2345);
        assert_eq!(p.p2pool_difficulty.to_string(), "10,000,000");
        assert_eq!(p.p2pool_hashrate.to_string(), "1.000 MH/s");
        assert_eq!(p.miners.to_string(), "1,000");
        assert_eq!(
            p.solo_block_mean.to_string(),
            "5 months, 21 days, 9 hours, 52 minutes"
        );
        assert_eq!(
            p.p2pool_block_mean.to_string(),
            "3 days, 11 hours, 20 minutes"
        );
        assert_eq!(p.p2pool_share_mean.to_string(), "8 minutes, 20 seconds");
        assert_eq!(p.p2pool_percent.to_string(), "0.040000%");
        assert_eq!(p.user_p2pool_percent.to_string(), "2.000000%");
        assert_eq!(p.user_monero_percent.to_string(), "0.000800%");
        drop(p);
    }

    #[test]
    fn set_xmrig_mining() {
        use crate::helper::PubXmrigApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubXmrigApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            "[2022-02-12 12:49:30.311]  net      no active pools, stop mining",
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::Xmrig,
            "".to_string(),
            PathBuf::new(),
        )));

        process.lock().unwrap().state = ProcessState::Alive;
        PubXmrigApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::NotMining);

        let output_parse = Arc::new(Mutex::new(String::from("[2022-02-12 12:49:30.311]  net      new job from 192.168.2.1:3333 diff 402K algo rx/0 height 2241142 (11 tx)")));
        PubXmrigApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn serde_priv_p2pool_local_api() {
        let data = r#"{
				"hashrate_15m": 12,
				"hashrate_1h": 11111,
				"hashrate_24h": 468967,
				"total_hashes": 2019283840922394082390,
				"shares_found": 289037,
				"average_effort": 915.563,
				"current_effort": 129.297,
				"connections": 123,
				"incoming_connections": 96
			}"#;
        let priv_api = PrivP2poolLocalApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "hashrate_15m": 12,
  "hashrate_1h": 11111,
  "hashrate_24h": 468967,
  "shares_found": 289037,
  "average_effort": 915.563,
  "current_effort": 129.297,
  "connections": 123
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_p2pool_network_api() {
        let data = r#"{
				"difficulty": 319028180924,
				"hash": "22ae1b83d727bb2ff4efc17b485bc47bc8bf5e29a7b3af65baf42213ac70a39b",
				"height": 2776576,
				"reward": 600499860000,
				"timestamp": 1670953659
			}"#;
        let priv_api = PrivP2poolNetworkApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "difficulty": 319028180924,
  "hash": "22ae1b83d727bb2ff4efc17b485bc47bc8bf5e29a7b3af65baf42213ac70a39b",
  "height": 2776576,
  "reward": 600499860000,
  "timestamp": 1670953659
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_p2pool_pool_api() {
        let data = r#"{
				"pool_list": ["pplns"],
				"pool_statistics": {
					"hashRate": 10225772,
					"miners": 713,
					"totalHashes": 487463929193948,
					"lastBlockFoundTime": 1670453228,
					"lastBlockFound": 2756570,
					"totalBlocksFound": 4
				}
			}"#;
        let priv_api = crate::helper::p2pool::PrivP2poolPoolApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "pool_statistics": {
    "hashRate": 10225772,
    "miners": 713
  }
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_xmrig_api() {
        let data = r#"{
		    "id": "6226e3sd0cd1a6es",
		    "worker_id": "hinto",
		    "uptime": 123,
		    "restricted": true,
		    "resources": {
		        "memory": {
		            "free": 123,
		            "total": 123123,
		            "resident_set_memory": 123123123
		        },
		        "load_average": [10.97, 10.58, 10.47],
		        "hardware_concurrency": 12
		    },
		    "features": ["api", "asm", "http", "hwloc", "tls", "opencl", "cuda"],
		    "results": {
		        "diff_current": 123,
		        "shares_good": 123,
		        "shares_total": 123,
		        "avg_time": 123,
		        "avg_time_ms": 123,
		        "hashes_total": 123,
		        "best": [123, 123, 123, 13, 123, 123, 123, 123, 123, 123],
		        "error_log": []
		    },
		    "algo": "rx/0",
		    "connection": {
		        "pool": "localhost:3333",
		        "ip": "127.0.0.1",
		        "uptime": 123,
		        "uptime_ms": 123,
		        "ping": 0,
		        "failures": 0,
		        "tls": null,
		        "tls-fingerprint": null,
		        "algo": "rx/0",
		        "diff": 123,
		        "accepted": 123,
		        "rejected": 123,
		        "avg_time": 123,
		        "avg_time_ms": 123,
		        "hashes_total": 123,
		        "error_log": []
		    },
		    "version": "6.18.0",
		    "kind": "miner",
		    "ua": "XMRig/6.18.0 (Linux x86_64) libuv/2.0.0-dev gcc/10.2.1",
		    "cpu": {
		        "brand": "blah blah blah",
		        "family": 1,
		        "model": 2,
		        "stepping": 0,
		        "proc_info": 123,
		        "aes": true,
		        "avx2": true,
		        "x64": true,
		        "64_bit": true,
		        "l2": 123123,
		        "l3": 123123,
		        "cores": 12,
		        "threads": 24,
		        "packages": 1,
		        "nodes": 1,
		        "backend": "hwloc/2.8.0a1-git",
		        "msr": "ryzen_19h",
		        "assembly": "ryzen",
		        "arch": "x86_64",
		        "flags": ["aes", "vaes", "avx", "avx2", "bmi2", "osxsave", "pdpe1gb", "sse2", "ssse3", "sse4.1", "popcnt", "cat_l3"]
		    },
		    "donate_level": 0,
		    "paused": false,
		    "algorithms": ["cn/1", "cn/2", "cn/r", "cn/fast", "cn/half", "cn/xao", "cn/rto", "cn/rwz", "cn/zls", "cn/double", "cn/ccx", "cn-lite/1", "cn-heavy/0", "cn-heavy/tube", "cn-heavy/xhv", "cn-pico", "cn-pico/tlo", "cn/upx2", "rx/0", "rx/wow", "rx/arq", "rx/graft", "rx/sfx", "rx/keva", "argon2/chukwa", "argon2/chukwav2", "argon2/ninja", "astrobwt", "astrobwt/v2", "ghostrider"],
		    "hashrate": {
		        "total": [111.11, 111.11, 111.11],
		        "highest": 111.11,
		        "threads": [
		            [111.11, 111.11, 111.11]
		        ]
		    },
		    "hugepages": true
		}"#;
        use crate::helper::xmrig::PrivXmrigApi;
        let priv_api = serde_json::from_str::<PrivXmrigApi>(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "worker_id": "hinto",
  "resources": {
    "load_average": [
      10.97,
      10.58,
      10.47
    ]
  },
  "connection": {
    "diff": 123,
    "accepted": 123,
    "rejected": 123
  },
  "hashrate": {
    "total": [
      111.11,
      111.11,
      111.11
    ]
  }
}"#;
        assert_eq!(data_after_ser, json)
    }
}
