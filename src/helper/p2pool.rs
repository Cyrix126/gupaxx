use super::Helper;
use super::Process;
use crate::components::node::RemoteNode;
use crate::disk::state::P2pool;
use crate::helper::check_died;
use crate::helper::check_user_input;
use crate::helper::signal_end;
use crate::helper::sleep_end_loop;
use crate::helper::ProcessName;
use crate::helper::ProcessSignal;
use crate::helper::ProcessState;
use crate::regex::contains_end_status;
use crate::regex::contains_statuscommand;
use crate::regex::contains_yourhashrate;
use crate::regex::contains_yourshare;
use crate::regex::estimated_hr;
use crate::regex::nb_current_shares;
use crate::regex::P2POOL_REGEX;
use crate::{
    constants::*,
    disk::{gupax_p2pool_api::GupaxP2poolApi, node::Node},
    helper::{MONERO_BLOCK_TIME_IN_SECONDS, P2POOL_BLOCK_TIME_IN_SECONDS},
    human::*,
    macros::*,
    xmr::*,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    fmt::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::*,
};
impl Helper {
    #[cold]
    #[inline(never)]
    fn read_pty_p2pool(
        output_parse: Arc<Mutex<String>>,
        output_pub: Arc<Mutex<String>>,
        reader: Box<dyn std::io::Read + Send>,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
        gui_api: Arc<Mutex<PubP2poolApi>>,
    ) {
        use std::io::BufRead;
        let mut stdout = std::io::BufReader::new(reader).lines();

        // Run a ANSI escape sequence filter for the first few lines.
        let mut i = 0;
        while let Some(Ok(line)) = stdout.next() {
            let line = strip_ansi_escapes::strip_str(line);

            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("P2Pool PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("P2Pool PTY Pub | Output error: {}", e);
            }
            if i > 20 {
                break;
            } else {
                i += 1;
            }
        }
        let mut status_output = false;
        while let Some(Ok(line)) = stdout.next() {
            // if command status is sent by gupaxx process and not the user, forward it only to update_from_status method.
            // 25 lines after the command are the result of status, with last line finishing by update.
            if contains_statuscommand(&line) {
                status_output = true;
                continue;
            }
            if status_output {
                if contains_yourhashrate(&line) {
                    if let Some(ehr) = estimated_hr(&line) {
                        debug!(
                            "P2pool | PTY getting current estimated HR data from status: {} KH/s",
                            ehr
                        );
                        // multiply by a thousand because value is given as kH/s instead H/s
                        lock!(gui_api).sidechain_ehr = ehr;
                        debug!(
                            "P2pool | PTY getting current estimated HR data from status: {} H/s",
                            lock!(gui_api).sidechain_ehr
                        );
                    } else {
                        error!("P2pool | PTY Getting data from status: Lines contains Your shares but no value found: {}", line);
                    }
                }
                if contains_yourshare(&line) {
                    // update sidechain shares
                    if let Some(shares) = nb_current_shares(&line) {
                        debug!(
                            "P2pool | PTY getting current shares data from status: {} share",
                            shares
                        );
                        lock!(gui_api).sidechain_shares = shares;
                    } else {
                        error!("P2pool | PTY Getting data from status: Lines contains Your shares but no value found: {}", line);
                    }
                }
                if contains_end_status(&line) {
                    // end of status
                    status_output = false;
                }
                continue;
            }
            //			println!("{}", line); // For debugging.
            if P2POOL_REGEX.payout.is_match(&line) {
                debug!("P2Pool PTY | Found payout, attempting write: {}", line);
                let (date, atomic_unit, block) = PayoutOrd::parse_raw_payout_line(&line);
                let formatted_log_line = GupaxP2poolApi::format_payout(&date, &atomic_unit, &block);
                GupaxP2poolApi::add_payout(
                    &mut lock!(gupax_p2pool_api),
                    &formatted_log_line,
                    date,
                    atomic_unit,
                    block,
                );
                if let Err(e) = GupaxP2poolApi::write_to_all_files(
                    &lock!(gupax_p2pool_api),
                    &formatted_log_line,
                ) {
                    error!("P2Pool PTY GupaxP2poolApi | Write error: {}", e);
                }
            }
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("P2Pool PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("P2Pool PTY Pub | Output error: {}", e);
            }
        }
    }
    //---------------------------------------------------------------------------------------------------- P2Pool specific
    #[cold]
    #[inline(never)]
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_p2pool(helper: &Arc<Mutex<Self>>) {
        info!("P2Pool | Attempting to stop...");
        lock2!(helper, p2pool).signal = ProcessSignal::Stop;
        lock2!(helper, p2pool).state = ProcessState::Middle;
    }

    #[cold]
    #[inline(never)]
    // The "restart frontend" to a "frontend" function.
    // Basically calls to kill the current p2pool, waits a little, then starts the below function in a a new thread, then exit.
    pub fn restart_p2pool(
        helper: &Arc<Mutex<Self>>,
        state: &P2pool,
        path: &Path,
        backup_hosts: Option<Vec<Node>>,
    ) {
        info!("P2Pool | Attempting to restart...");
        lock2!(helper, p2pool).signal = ProcessSignal::Restart;
        lock2!(helper, p2pool).state = ProcessState::Middle;

        let helper = Arc::clone(helper);
        let state = state.clone();
        let path = path.to_path_buf();
        // This thread lives to wait, start p2pool then die.
        thread::spawn(move || {
            while lock2!(helper, p2pool).is_alive() {
                warn!("P2Pool | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("P2Pool | Old process seems dead, starting new one!");
            Self::start_p2pool(&helper, &state, &path, backup_hosts);
        });
        info!("P2Pool | Restart ... OK");
    }

    #[cold]
    #[inline(never)]
    // The "frontend" function that parses the arguments, and spawns either the [Simple] or [Advanced] P2Pool watchdog thread.
    pub fn start_p2pool(
        helper: &Arc<Mutex<Self>>,
        state: &P2pool,
        path: &Path,
        backup_hosts: Option<Vec<Node>>,
    ) {
        lock2!(helper, p2pool).state = ProcessState::Middle;

        let (args, api_path_local, api_path_network, api_path_pool) =
            Self::build_p2pool_args_and_mutate_img(helper, state, path, backup_hosts);

        // Print arguments & user settings to console
        crate::disk::print_dash(&format!(
			"P2Pool | Launch arguments: {:#?} | Local API Path: {:#?} | Network API Path: {:#?} | Pool API Path: {:#?}",
			 args,
			 api_path_local,
			 api_path_network,
			 api_path_pool,
		));

        // Spawn watchdog thread
        let process = Arc::clone(&lock!(helper).p2pool);
        let gui_api = Arc::clone(&lock!(helper).gui_api_p2pool);
        let pub_api = Arc::clone(&lock!(helper).pub_api_p2pool);
        let gupax_p2pool_api = Arc::clone(&lock!(helper).gupax_p2pool_api);
        let path = path.to_path_buf();
        thread::spawn(move || {
            Self::spawn_p2pool_watchdog(
                process,
                gui_api,
                pub_api,
                args,
                path,
                api_path_local,
                api_path_network,
                api_path_pool,
                gupax_p2pool_api,
            );
        });
    }
    // Takes in a 95-char Monero address, returns the first and last
    // 8 characters separated with dots like so: [4abcdefg...abcdefgh]
    pub fn head_tail_of_monero_address(address: &str) -> String {
        if address.len() < 95 {
            return "???".to_string();
        }
        let head = &address[0..8];
        let tail = &address[87..95];
        head.to_owned() + "..." + tail
    }

    #[cold]
    #[inline(never)]
    // Takes in some [State/P2pool] and parses it to build the actual command arguments.
    // Returns the [Vec] of actual arguments, and mutates the [ImgP2pool] for the main GUI thread
    // It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
    pub fn build_p2pool_args_and_mutate_img(
        helper: &Arc<Mutex<Self>>,
        state: &P2pool,
        path: &Path,
        backup_hosts: Option<Vec<Node>>,
    ) -> (Vec<String>, PathBuf, PathBuf, PathBuf) {
        let mut args = Vec::with_capacity(500);
        let path = path.to_path_buf();
        let mut api_path = path;
        api_path.pop();

        // [Simple]
        if state.simple {
            // Build the p2pool argument
            let (ip, rpc, zmq) = RemoteNode::get_ip_rpc_zmq(&state.node); // Get: (IP, RPC, ZMQ)
            args.push("--wallet".to_string());
            args.push(state.address.clone()); // Wallet address
            args.push("--host".to_string());
            args.push(ip.to_string()); // IP Address
            args.push("--rpc-port".to_string());
            args.push(rpc.to_string()); // RPC Port
            args.push("--zmq-port".to_string());
            args.push(zmq.to_string()); // ZMQ Port
            args.push("--data-api".to_string());
            args.push(api_path.display().to_string()); // API Path
            args.push("--local-api".to_string()); // Enable API
            args.push("--no-color".to_string()); // Remove color escape sequences, Gupax terminal can't parse it :(
            args.push("--mini".to_string()); // P2Pool Mini
            args.push("--light-mode".to_string()); // Assume user is not using P2Pool to mine.

            // Push other nodes if `backup_host`.
            if let Some(nodes) = backup_hosts {
                for node in nodes {
                    if (node.ip.as_str(), node.rpc.as_str(), node.zmq.as_str()) != (ip, rpc, zmq) {
                        args.push("--host".to_string());
                        args.push(node.ip.to_string());
                        args.push("--rpc-port".to_string());
                        args.push(node.rpc.to_string());
                        args.push("--zmq-port".to_string());
                        args.push(node.zmq.to_string());
                    }
                }
            }

            *lock2!(helper, img_p2pool) = ImgP2pool {
                mini: "P2Pool Mini".to_string(),
                address: Self::head_tail_of_monero_address(&state.address),
                host: ip.to_string(),
                rpc: rpc.to_string(),
                zmq: zmq.to_string(),
                out_peers: "10".to_string(),
                in_peers: "10".to_string(),
            };

        // [Advanced]
        } else {
            // Overriding command arguments
            if !state.arguments.is_empty() {
                // This parses the input and attempts to fill out
                // the [ImgP2pool]... This is pretty bad code...
                let mut last = "";
                let lock = lock!(helper);
                let mut p2pool_image = lock!(lock.img_p2pool);
                let mut mini = false;
                for arg in state.arguments.split_whitespace() {
                    match last {
                        "--mini" => {
                            mini = true;
                            p2pool_image.mini = "P2Pool Mini".to_string();
                        }
                        "--wallet" => p2pool_image.address = Self::head_tail_of_monero_address(arg),
                        "--host" => p2pool_image.host = arg.to_string(),
                        "--rpc-port" => p2pool_image.rpc = arg.to_string(),
                        "--zmq-port" => p2pool_image.zmq = arg.to_string(),
                        "--out-peers" => p2pool_image.out_peers = arg.to_string(),
                        "--in-peers" => p2pool_image.in_peers = arg.to_string(),
                        "--data-api" => api_path = PathBuf::from(arg),
                        _ => (),
                    }
                    if !mini {
                        p2pool_image.mini = "P2Pool Main".to_string();
                    }
                    let arg = if arg == "localhost" { "127.0.0.1" } else { arg };
                    args.push(arg.to_string());
                    last = arg;
                }
            // Else, build the argument
            } else {
                let ip = if state.ip == "localhost" {
                    "127.0.0.1"
                } else {
                    &state.ip
                };
                args.push("--wallet".to_string());
                args.push(state.address.clone()); // Wallet
                args.push("--host".to_string());
                args.push(ip.to_string()); // IP
                args.push("--rpc-port".to_string());
                args.push(state.rpc.to_string()); // RPC
                args.push("--zmq-port".to_string());
                args.push(state.zmq.to_string()); // ZMQ
                args.push("--loglevel".to_string());
                args.push(state.log_level.to_string()); // Log Level
                args.push("--out-peers".to_string());
                args.push(state.out_peers.to_string()); // Out Peers
                args.push("--in-peers".to_string());
                args.push(state.in_peers.to_string()); // In Peers
                args.push("--data-api".to_string());
                args.push(api_path.display().to_string()); // API Path
                args.push("--local-api".to_string()); // Enable API
                args.push("--no-color".to_string()); // Remove color escape sequences
                args.push("--light-mode".to_string()); // Assume user is not using P2Pool to mine.
                if state.mini {
                    args.push("--mini".to_string());
                }; // Mini

                // Push other nodes if `backup_host`.
                if let Some(nodes) = backup_hosts {
                    for node in nodes {
                        let ip = if node.ip == "localhost" {
                            "127.0.0.1"
                        } else {
                            &node.ip
                        };
                        if (node.ip.as_str(), node.rpc.as_str(), node.zmq.as_str())
                            != (ip, &state.rpc, &state.zmq)
                        {
                            args.push("--host".to_string());
                            args.push(node.ip.to_string());
                            args.push("--rpc-port".to_string());
                            args.push(node.rpc.to_string());
                            args.push("--zmq-port".to_string());
                            args.push(node.zmq.to_string());
                        }
                    }
                }

                *lock2!(helper, img_p2pool) = ImgP2pool {
                    mini: if state.mini {
                        "P2Pool Mini".to_string()
                    } else {
                        "P2Pool Main".to_string()
                    },
                    address: Self::head_tail_of_monero_address(&state.address),
                    host: state.selected_ip.to_string(),
                    rpc: state.selected_rpc.to_string(),
                    zmq: state.selected_zmq.to_string(),
                    out_peers: state.out_peers.to_string(),
                    in_peers: state.in_peers.to_string(),
                };
            }
        }
        let mut api_path_local = api_path.clone();
        let mut api_path_network = api_path.clone();
        let mut api_path_pool = api_path.clone();
        api_path_local.push(P2POOL_API_PATH_LOCAL);
        api_path_network.push(P2POOL_API_PATH_NETWORK);
        api_path_pool.push(P2POOL_API_PATH_POOL);
        (args, api_path_local, api_path_network, api_path_pool)
    }

    #[cold]
    #[inline(never)]
    // The P2Pool watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
    // or if P2Pool simple is false and extern is true, only prints data from stratum api.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::await_holding_lock)]
    #[tokio::main]
    async fn spawn_p2pool_watchdog(
        process: Arc<Mutex<Process>>,
        gui_api: Arc<Mutex<PubP2poolApi>>,
        pub_api: Arc<Mutex<PubP2poolApi>>,
        args: Vec<String>,
        path: std::path::PathBuf,
        api_path_local: std::path::PathBuf,
        api_path_network: std::path::PathBuf,
        api_path_pool: std::path::PathBuf,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
    ) {
        // 1a. Create PTY
        debug!("P2Pool | Creating PTY...");
        let pty = portable_pty::native_pty_system();
        let pair = pty
            .openpty(portable_pty::PtySize {
                rows: 100,
                cols: 1000,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        // 1b. Create command
        debug!("P2Pool | Creating command...");
        let mut cmd = portable_pty::CommandBuilder::new(path.as_path());
        cmd.args(args);
        cmd.env("NO_COLOR", "true");
        cmd.cwd(path.as_path().parent().unwrap());
        // 1c. Create child
        debug!("P2Pool | Creating child...");
        let child_pty = arc_mut!(pair.slave.spawn_command(cmd).unwrap());
        drop(pair.slave);

        // 2. Set process state
        debug!("P2Pool | Setting process state...");
        let mut lock = lock!(process);
        lock.state = ProcessState::Syncing;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
        let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
        let mut stdin = pair.master.take_writer().unwrap();
        drop(lock);

        // 3. Spawn PTY read thread
        debug!("P2Pool | Spawning PTY read thread...");
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);
        let gupax_p2pool_api = Arc::clone(&gupax_p2pool_api);
        let p2pool_api_c = Arc::clone(&gui_api);
        tokio::spawn(async move {
            Self::read_pty_p2pool(
                output_parse,
                output_pub,
                reader,
                gupax_p2pool_api,
                p2pool_api_c,
            );
        });
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);

        debug!("P2Pool | Cleaning old [local] API files...");
        // Attempt to remove stale API file
        match std::fs::remove_file(&api_path_local) {
            Ok(_) => info!("P2Pool | Attempting to remove stale API file ... OK"),
            Err(e) => warn!(
                "P2Pool | Attempting to remove stale API file ... FAIL ... {}",
                e
            ),
        }
        // Attempt to create a default empty one.
        use std::io::Write;
        if std::fs::File::create(&api_path_local).is_ok() {
            let text = r#"{"hashrate_15m":0,"hashrate_1h":0,"hashrate_24h":0,"shares_found":0,"average_effort":0.0,"current_effort":0.0,"connections":0}"#;
            match std::fs::write(&api_path_local, text) {
                Ok(_) => info!("P2Pool | Creating default empty API file ... OK"),
                Err(e) => warn!(
                    "P2Pool | Creating default empty API file ... FAIL ... {}",
                    e
                ),
            }
        }
        let start = lock!(process).start;

        // Reset stats before loop
        *lock!(pub_api) = PubP2poolApi::new();
        *lock!(gui_api) = PubP2poolApi::new();

        // 4. Loop as watchdog
        let mut first_loop = true;
        let mut last_p2pool_request = tokio::time::Instant::now();
        let mut last_status_request = tokio::time::Instant::now();

        info!("P2Pool | Entering watchdog mode... woof!");
        loop {
            // Set timer
            let now = Instant::now();
            debug!("P2Pool Watchdog | ----------- Start of loop -----------");
            lock!(gui_api).tick = (last_p2pool_request.elapsed().as_secs() % 60) as u8;

            // Check if the process is secretly died without us knowing :)
            if check_died(
                &child_pty,
                &mut lock!(process),
                &start,
                &mut lock!(gui_api).output,
            ) {
                break;
            }

            // Check SIGNAL
            if signal_end(&process, &child_pty, &start, &mut lock!(gui_api).output) {
                break;
            }
            // Check vector of user input
            check_user_input(&process, &mut stdin);
            // Check if logs need resetting
            debug!("P2Pool Watchdog | Attempting GUI log reset check");
            let mut lock = lock!(gui_api);
            Self::check_reset_gui_output(&mut lock.output, ProcessName::P2pool);
            drop(lock);

            // Always update from output
            debug!("P2Pool Watchdog | Starting [update_from_output()]");
            PubP2poolApi::update_from_output(
                &pub_api,
                &output_parse,
                &output_pub,
                start.elapsed(),
                &process,
            );

            // Read [local] API
            debug!("P2Pool Watchdog | Attempting [local] API file read");
            if let Ok(string) = Self::path_to_string(&api_path_local, ProcessName::P2pool) {
                // Deserialize
                if let Ok(local_api) = PrivP2poolLocalApi::from_str(&string) {
                    // Update the structs.
                    PubP2poolApi::update_from_local(&pub_api, local_api);
                }
            }
            // If more than 1 minute has passed, read the other API files.
            let last_p2pool_request_expired =
                last_p2pool_request.elapsed() >= Duration::from_secs(60);

            if last_p2pool_request_expired {
                debug!("P2Pool Watchdog | Attempting [network] & [pool] API file read");
                if let (Ok(network_api), Ok(pool_api)) = (
                    Self::path_to_string(&api_path_network, ProcessName::P2pool),
                    Self::path_to_string(&api_path_pool, ProcessName::P2pool),
                ) {
                    if let (Ok(network_api), Ok(pool_api)) = (
                        PrivP2poolNetworkApi::from_str(&network_api),
                        PrivP2poolPoolApi::from_str(&pool_api),
                    ) {
                        PubP2poolApi::update_from_network_pool(&pub_api, network_api, pool_api);
                        last_p2pool_request = tokio::time::Instant::now();
                    }
                }
            }

            let last_status_request_expired =
                last_status_request.elapsed() >= Duration::from_secs(60);

            if (last_status_request_expired || first_loop)
                && lock!(process).state == ProcessState::Alive
            {
                debug!("P2Pool Watchdog | Reading status output of p2pool node");
                #[cfg(target_os = "windows")]
                if let Err(e) = write!(stdin, "statusfromgupaxx\r\n") {
                    error!("P2Pool Watchdog | STDIN error: {}", e);
                }
                #[cfg(target_family = "unix")]
                if let Err(e) = writeln!(stdin, "statusfromgupaxx") {
                    error!("P2Pool Watchdog | STDIN error: {}", e);
                }
                // Flush.
                if let Err(e) = stdin.flush() {
                    error!("P2Pool Watchdog | STDIN flush error: {}", e);
                }
                last_status_request = tokio::time::Instant::now();
            }

            // Sleep (only if 900ms hasn't passed)
            if first_loop {
                first_loop = false;
            }
            sleep_end_loop(now, ProcessName::P2pool).await;
            debug!(
                "P2Pool Watchdog | END OF LOOP -  Tick: [{}/60]",
                lock!(gui_api).tick,
            );
        }

        // 5. If loop broke, we must be done here.
        info!("P2Pool Watchdog | Watchdog thread exiting... Goodbye!");
    }
}
//---------------------------------------------------------------------------------------------------- [ImgP2pool]
// A static "image" of data that P2Pool started with.
// This is just a snapshot of the user data when they initially started P2Pool.
// Created by [start_p2pool()] and return to the main GUI thread where it will store it.
// No need for an [Arc<Mutex>] since the Helper thread doesn't need this information.
#[derive(Debug, Clone)]
pub struct ImgP2pool {
    pub mini: String,      // Did the user start on the mini-chain?
    pub address: String, // What address is the current p2pool paying out to? (This gets shortened to [4xxxxx...xxxxxx])
    pub host: String,    // What monerod are we using?
    pub rpc: String,     // What is the RPC port?
    pub zmq: String,     // What is the ZMQ port?
    pub out_peers: String, // How many out-peers?
    pub in_peers: String, // How many in-peers?
}

impl Default for ImgP2pool {
    fn default() -> Self {
        Self::new()
    }
}

impl ImgP2pool {
    pub fn new() -> Self {
        Self {
            mini: String::from("???"),
            address: String::from("???"),
            host: String::from("???"),
            rpc: String::from("???"),
            zmq: String::from("???"),
            out_peers: String::from("???"),
            in_peers: String::from("???"),
        }
    }
}

//---------------------------------------------------------------------------------------------------- Public P2Pool API
// Helper/GUI threads both have a copy of this, Helper updates
// the GUI's version on a 1-second interval from the private data.
#[derive(Debug, Clone, PartialEq)]
pub struct PubP2poolApi {
    // Output
    pub output: String,
    // Uptime
    pub uptime: HumanTime,
    // These are manually parsed from the STDOUT.
    pub payouts: u128,
    pub payouts_hour: f64,
    pub payouts_day: f64,
    pub payouts_month: f64,
    pub xmr: f64,
    pub xmr_hour: f64,
    pub xmr_day: f64,
    pub xmr_month: f64,
    // Local API
    pub hashrate_15m: HumanNumber,
    pub hashrate_1h: HumanNumber,
    pub hashrate_24h: HumanNumber,
    pub shares_found: Option<u64>,
    pub average_effort: HumanNumber,
    pub current_effort: HumanNumber,
    pub connections: HumanNumber,
    // The API needs a raw ints to go off of and
    // there's not a good way to access it without doing weird
    // [Arc<Mutex>] shenanigans, so some raw ints are stored here.
    pub user_p2pool_hashrate_u64: u64,
    pub p2pool_difficulty_u64: u64,
    pub monero_difficulty_u64: u64,
    pub p2pool_hashrate_u64: u64,
    pub monero_hashrate_u64: u64,
    // Tick. Every loop this gets incremented.
    // At 60, it indicated we should read the below API files.
    pub tick: u8,
    // Network API
    pub monero_difficulty: HumanNumber, // e.g: [15,000,000]
    pub monero_hashrate: HumanNumber,   // e.g: [1.000 GH/s]
    pub hash: String,                   // Current block hash
    pub height: HumanNumber,
    pub reward: AtomicUnit,
    // Pool API
    pub p2pool_difficulty: HumanNumber,
    pub p2pool_hashrate: HumanNumber,
    pub miners: HumanNumber, // Current amount of miners on P2Pool sidechain
    // Mean (calculated in functions, not serialized)
    pub solo_block_mean: HumanTime, // Time it would take the user to find a solo block
    pub p2pool_block_mean: HumanTime, // Time it takes the P2Pool sidechain to find a block
    pub p2pool_share_mean: HumanTime, // Time it would take the user to find a P2Pool share
    // Percent
    pub p2pool_percent: HumanNumber, // Percentage of P2Pool hashrate capture of overall Monero hashrate.
    pub user_p2pool_percent: HumanNumber, // How much percent the user's hashrate accounts for in P2Pool.
    pub user_monero_percent: HumanNumber, // How much percent the user's hashrate accounts for in all of Monero hashrate.
    // from status
    pub sidechain_shares: u32,
    pub sidechain_ehr: f32,
}

impl Default for PubP2poolApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PubP2poolApi {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            uptime: HumanTime::new(),
            payouts: 0,
            payouts_hour: 0.0,
            payouts_day: 0.0,
            payouts_month: 0.0,
            xmr: 0.0,
            xmr_hour: 0.0,
            xmr_day: 0.0,
            xmr_month: 0.0,
            hashrate_15m: HumanNumber::unknown(),
            hashrate_1h: HumanNumber::unknown(),
            hashrate_24h: HumanNumber::unknown(),
            shares_found: None,
            average_effort: HumanNumber::unknown(),
            current_effort: HumanNumber::unknown(),
            connections: HumanNumber::unknown(),
            tick: 0,
            user_p2pool_hashrate_u64: 0,
            p2pool_difficulty_u64: 0,
            monero_difficulty_u64: 0,
            p2pool_hashrate_u64: 0,
            monero_hashrate_u64: 0,
            monero_difficulty: HumanNumber::unknown(),
            monero_hashrate: HumanNumber::unknown(),
            hash: String::from("???"),
            height: HumanNumber::unknown(),
            reward: AtomicUnit::new(),
            p2pool_difficulty: HumanNumber::unknown(),
            p2pool_hashrate: HumanNumber::unknown(),
            miners: HumanNumber::unknown(),
            solo_block_mean: HumanTime::new(),
            p2pool_block_mean: HumanTime::new(),
            p2pool_share_mean: HumanTime::new(),
            p2pool_percent: HumanNumber::unknown(),
            user_p2pool_percent: HumanNumber::unknown(),
            user_monero_percent: HumanNumber::unknown(),
            sidechain_shares: 0,
            sidechain_ehr: 0.0,
        }
    }

    #[inline]
    // The issue with just doing [gui_api = pub_api] is that values get overwritten.
    // This doesn't matter for any of the values EXCEPT for the output, so we must
    // manually append it instead of overwriting.
    // This is used in the "helper" thread.
    pub(super) fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        let mut output = std::mem::take(&mut gui_api.output);
        let buf = std::mem::take(&mut pub_api.output);
        if !buf.is_empty() {
            output.push_str(&buf);
        }
        *gui_api = Self {
            output,
            tick: std::mem::take(&mut gui_api.tick),
            sidechain_shares: std::mem::take(&mut gui_api.sidechain_shares),
            sidechain_ehr: std::mem::take(&mut gui_api.sidechain_ehr),
            ..pub_api.clone()
        };
    }

    #[inline]
    // Essentially greps the output for [x.xxxxxxxxxxxx XMR] where x = a number.
    // It sums each match and counts along the way, handling an error by not adding and printing to console.
    fn calc_payouts_and_xmr(output: &str) -> (u128 /* payout count */, f64 /* total xmr */) {
        let iter = P2POOL_REGEX.payout.find_iter(output);
        let mut sum: f64 = 0.0;
        let mut count: u128 = 0;
        for i in iter {
            if let Some(word) = P2POOL_REGEX.payout_float.find(i.as_str()) {
                match word.as_str().parse::<f64>() {
                    Ok(num) => {
                        sum += num;
                        count += 1;
                    }
                    Err(e) => error!("P2Pool | Total XMR sum calculation error: [{}]", e),
                }
            }
        }
        (count, sum)
    }

    // Mutate "watchdog"'s [PubP2poolApi] with data the process output.
    pub(super) fn update_from_output(
        public: &Arc<Mutex<Self>>,
        output_parse: &Arc<Mutex<String>>,
        output_pub: &Arc<Mutex<String>>,
        elapsed: std::time::Duration,
        process: &Arc<Mutex<Process>>,
    ) {
        // 1. Take the process's current output buffer and combine it with Pub (if not empty)
        let mut output_pub = lock!(output_pub);
        if !output_pub.is_empty() {
            lock!(public)
                .output
                .push_str(&std::mem::take(&mut *output_pub));
        }

        // 2. Parse the full STDOUT
        let mut output_parse = lock!(output_parse);
        let (payouts_new, xmr_new) = Self::calc_payouts_and_xmr(&output_parse);
        // Check for "SYNCHRONIZED" only if we aren't already.
        if lock!(process).state == ProcessState::Syncing {
            // How many times the word was captured.
            let synchronized_captures = P2POOL_REGEX.synchronized.find_iter(&output_parse).count();

            // If P2Pool receives shares before syncing, it will start mining on its own sidechain.
            // In this instance, we technically are "synced" on block 1 and P2Pool will print "SYNCHRONIZED"
            // although, that doesn't necessarily mean we're synced on main/mini-chain.
            //
            // So, if we find a `next block = 1`, that means we
            // must look for at least 2 instances of "SYNCHRONIZED",
            // one for the sidechain, one for main/mini.
            if P2POOL_REGEX.next_height_1.is_match(&output_parse) {
                if synchronized_captures > 1 {
                    lock!(process).state = ProcessState::Alive;
                }
            } else if synchronized_captures > 0 {
                // if there is no `next block = 1`, fallback to
                // just finding 1 instance of "SYNCHRONIZED".
                lock!(process).state = ProcessState::Alive;
            }
        }
        // 3. Throw away [output_parse]
        output_parse.clear();
        drop(output_parse);
        // 4. Add to current values
        let mut public = lock!(public);
        let (payouts, xmr) = (public.payouts + payouts_new, public.xmr + xmr_new);

        // 5. Calculate hour/day/month given elapsed time
        let elapsed_as_secs_f64 = elapsed.as_secs_f64();
        // Payouts
        let per_sec = (payouts as f64) / elapsed_as_secs_f64;
        let payouts_hour = (per_sec * 60.0) * 60.0;
        let payouts_day = payouts_hour * 24.0;
        let payouts_month = payouts_day * 30.0;
        // Total XMR
        let per_sec = xmr / elapsed_as_secs_f64;
        let xmr_hour = (per_sec * 60.0) * 60.0;
        let xmr_day = xmr_hour * 24.0;
        let xmr_month = xmr_day * 30.0;

        if payouts_new != 0 {
            debug!(
                "P2Pool Watchdog | New [Payout] found in output ... {}",
                payouts_new
            );
            debug!("P2Pool Watchdog | Total [Payout] should be ... {}", payouts);
            debug!(
                "P2Pool Watchdog | Correct [Payout per] should be ... [{}/hour, {}/day, {}/month]",
                payouts_hour, payouts_day, payouts_month
            );
        }
        if xmr_new != 0.0 {
            debug!(
                "P2Pool Watchdog | New [XMR mined] found in output ... {}",
                xmr_new
            );
            debug!("P2Pool Watchdog | Total [XMR mined] should be ... {}", xmr);
            debug!("P2Pool Watchdog | Correct [XMR mined per] should be ... [{}/hour, {}/day, {}/month]", xmr_hour, xmr_day, xmr_month);
        }

        // 6. Mutate the struct with the new info
        *public = Self {
            uptime: HumanTime::into_human(elapsed),
            payouts,
            xmr,
            payouts_hour,
            payouts_day,
            payouts_month,
            xmr_hour,
            xmr_day,
            xmr_month,
            ..std::mem::take(&mut *public)
        };
    }

    // Mutate [PubP2poolApi] with data from a [PrivP2poolLocalApi] and the process output.
    pub(super) fn update_from_local(public: &Arc<Mutex<Self>>, local: PrivP2poolLocalApi) {
        let mut public = lock!(public);
        *public = Self {
            hashrate_15m: HumanNumber::from_u64(local.hashrate_15m),
            hashrate_1h: HumanNumber::from_u64(local.hashrate_1h),
            hashrate_24h: HumanNumber::from_u64(local.hashrate_24h),
            shares_found: Some(local.shares_found),
            average_effort: HumanNumber::to_percent(local.average_effort),
            current_effort: HumanNumber::to_percent(local.current_effort),
            connections: HumanNumber::from_u32(local.connections),
            user_p2pool_hashrate_u64: local.hashrate_1h,
            ..std::mem::take(&mut *public)
        };
    }

    // Mutate [PubP2poolApi] with data from a [PrivP2pool(Network|Pool)Api].
    pub(super) fn update_from_network_pool(
        public: &Arc<Mutex<Self>>,
        net: PrivP2poolNetworkApi,
        pool: PrivP2poolPoolApi,
    ) {
        let user_hashrate = lock!(public).user_p2pool_hashrate_u64; // The user's total P2Pool hashrate
        let monero_difficulty = net.difficulty;
        let monero_hashrate = monero_difficulty / MONERO_BLOCK_TIME_IN_SECONDS;
        let p2pool_hashrate = pool.pool_statistics.hashRate;
        let p2pool_difficulty = p2pool_hashrate * P2POOL_BLOCK_TIME_IN_SECONDS;
        // These [0] checks prevent dividing by 0 (it [panic!()]s)
        let p2pool_block_mean;
        let user_p2pool_percent;
        if p2pool_hashrate == 0 {
            p2pool_block_mean = HumanTime::new();
            user_p2pool_percent = HumanNumber::unknown();
        } else {
            p2pool_block_mean = HumanTime::into_human(std::time::Duration::from_secs(
                monero_difficulty / p2pool_hashrate,
            ));
            let f = (user_hashrate as f64 / p2pool_hashrate as f64) * 100.0;
            user_p2pool_percent = HumanNumber::from_f64_to_percent_6_point(f);
        };
        let p2pool_percent;
        let user_monero_percent;
        if monero_hashrate == 0 {
            p2pool_percent = HumanNumber::unknown();
            user_monero_percent = HumanNumber::unknown();
        } else {
            let f = (p2pool_hashrate as f64 / monero_hashrate as f64) * 100.0;
            p2pool_percent = HumanNumber::from_f64_to_percent_6_point(f);
            let f = (user_hashrate as f64 / monero_hashrate as f64) * 100.0;
            user_monero_percent = HumanNumber::from_f64_to_percent_6_point(f);
        };
        let solo_block_mean;
        let p2pool_share_mean;
        if user_hashrate == 0 {
            solo_block_mean = HumanTime::new();
            p2pool_share_mean = HumanTime::new();
        } else {
            solo_block_mean = HumanTime::into_human(std::time::Duration::from_secs(
                monero_difficulty / user_hashrate,
            ));
            p2pool_share_mean = HumanTime::into_human(std::time::Duration::from_secs(
                p2pool_difficulty / user_hashrate,
            ));
        }
        let mut public = lock!(public);
        *public = Self {
            p2pool_difficulty_u64: p2pool_difficulty,
            monero_difficulty_u64: monero_difficulty,
            p2pool_hashrate_u64: p2pool_hashrate,
            monero_hashrate_u64: monero_hashrate,
            monero_difficulty: HumanNumber::from_u64(monero_difficulty),
            monero_hashrate: HumanNumber::from_u64_to_gigahash_3_point(monero_hashrate),
            hash: net.hash,
            height: HumanNumber::from_u32(net.height),
            reward: AtomicUnit::from_u64(net.reward),
            p2pool_difficulty: HumanNumber::from_u64(p2pool_difficulty),
            p2pool_hashrate: HumanNumber::from_u64_to_megahash_3_point(p2pool_hashrate),
            miners: HumanNumber::from_u32(pool.pool_statistics.miners),
            solo_block_mean,
            p2pool_block_mean,
            p2pool_share_mean,
            p2pool_percent,
            user_p2pool_percent,
            user_monero_percent,
            ..std::mem::take(&mut *public)
        };
    }

    #[inline]
    pub fn calculate_share_or_block_time(hashrate: u64, difficulty: u64) -> HumanTime {
        if hashrate == 0 {
            HumanTime::new()
        } else {
            HumanTime::from_u64(difficulty / hashrate)
        }
    }

    #[inline]
    pub fn calculate_dominance(my_hashrate: u64, global_hashrate: u64) -> HumanNumber {
        if global_hashrate == 0 {
            HumanNumber::unknown()
        } else {
            let f = (my_hashrate as f64 / global_hashrate as f64) * 100.0;
            HumanNumber::from_f64_to_percent_6_point(f)
        }
    }

    pub const fn calculate_tick_bar(&self) -> &'static str {
        // The stars are reduced by one because it takes a frame to render the stats.
        // We want 0 stars at the same time stats are rendered, so it looks a little off here.
        // let stars = "*".repeat(self.tick - 1);
        // let blanks = " ".repeat(60 - (self.tick - 1));
        // [use crate::PubP2poolApi;use crate::PubP2poolApi;"[", &stars, &blanks, "]"].concat().as_str()
        match self.tick {
            0 => "[                                                            ]",
            1 => "[*                                                           ]",
            2 => "[**                                                          ]",
            3 => "[***                                                         ]",
            4 => "[****                                                        ]",
            5 => "[*****                                                       ]",
            6 => "[******                                                      ]",
            7 => "[*******                                                     ]",
            8 => "[********                                                    ]",
            9 => "[*********                                                   ]",
            10 => "[**********                                                  ]",
            11 => "[***********                                                 ]",
            12 => "[************                                                ]",
            13 => "[*************                                               ]",
            14 => "[**************                                              ]",
            15 => "[***************                                             ]",
            16 => "[****************                                            ]",
            17 => "[*****************                                           ]",
            18 => "[******************                                          ]",
            19 => "[*******************                                         ]",
            20 => "[********************                                        ]",
            21 => "[*********************                                       ]",
            22 => "[**********************                                      ]",
            23 => "[***********************                                     ]",
            24 => "[************************                                    ]",
            25 => "[*************************                                   ]",
            26 => "[**************************                                  ]",
            27 => "[***************************                                 ]",
            28 => "[****************************                                ]",
            29 => "[*****************************                               ]",
            30 => "[******************************                              ]",
            31 => "[*******************************                             ]",
            32 => "[********************************                            ]",
            33 => "[*********************************                           ]",
            34 => "[**********************************                          ]",
            35 => "[***********************************                         ]",
            36 => "[************************************                        ]",
            37 => "[*************************************                       ]",
            38 => "[**************************************                      ]",
            39 => "[***************************************                     ]",
            40 => "[****************************************                    ]",
            41 => "[*****************************************                   ]",
            42 => "[******************************************                  ]",
            43 => "[*******************************************                 ]",
            44 => "[********************************************                ]",
            45 => "[*********************************************               ]",
            46 => "[**********************************************              ]",
            47 => "[***********************************************             ]",
            48 => "[************************************************            ]",
            49 => "[*************************************************           ]",
            50 => "[**************************************************          ]",
            51 => "[***************************************************         ]",
            52 => "[****************************************************        ]",
            53 => "[*****************************************************       ]",
            54 => "[******************************************************      ]",
            55 => "[*******************************************************     ]",
            56 => "[********************************************************    ]",
            57 => "[*********************************************************   ]",
            58 => "[**********************************************************  ]",
            59 => "[*********************************************************** ]",
            _ => "[************************************************************]",
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Local" Api
// This matches directly to P2Pool's [local/stratum] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(super) struct PrivP2poolLocalApi {
    pub hashrate_15m: u64,
    pub hashrate_1h: u64,
    pub hashrate_24h: u64,
    pub shares_found: u64,
    pub average_effort: f32,
    pub current_effort: f32,
    pub connections: u32, // This is a `uint32_t` in `p2pool`
}

impl Default for PrivP2poolLocalApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolLocalApi {
    fn new() -> Self {
        Self {
            hashrate_15m: 0,
            hashrate_1h: 0,
            hashrate_24h: 0,
            shares_found: 0,
            average_effort: 0.0,
            current_effort: 0.0,
            connections: 0,
        }
    }

    // Deserialize the above [String] into a [PrivP2poolApi]
    pub(super) fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Local API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Network" API
// This matches P2Pool's [network/stats] JSON API file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct PrivP2poolNetworkApi {
    pub difficulty: u64,
    pub hash: String,
    pub height: u32,
    pub reward: u64,
    pub timestamp: u32,
}

impl Default for PrivP2poolNetworkApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolNetworkApi {
    fn new() -> Self {
        Self {
            difficulty: 0,
            hash: String::from("???"),
            height: 0,
            reward: 0,
            timestamp: 0,
        }
    }

    pub(super) fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Network API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Pool" API
// This matches P2Pool's [pool/stats] JSON API file.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(super) struct PrivP2poolPoolApi {
    pub pool_statistics: PoolStatistics,
}

impl Default for PrivP2poolPoolApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolPoolApi {
    fn new() -> Self {
        Self {
            pool_statistics: PoolStatistics::new(),
        }
    }

    pub(super) fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Pool API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(super) struct PoolStatistics {
    pub hashRate: u64,
    pub miners: u32,
}
impl Default for PoolStatistics {
    fn default() -> Self {
        Self::new()
    }
}
impl PoolStatistics {
    fn new() -> Self {
        Self {
            hashRate: 0,
            miners: 0,
        }
    }
}
