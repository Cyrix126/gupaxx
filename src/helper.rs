// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janaiyo
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
// found here, e.g: User clicks [Start P2Pool] -> Arc<Mutex<ProcessSignal> is set
// indicating to this thread during its loop: "I should start P2Pool!", e.g:
//
//     match p2pool.lock().unwrap().signal {
//         ProcessSignal::Start => start_p2pool(),
//         ...
//     }
//
// This also includes all things related to handling the child processes (P2Pool/XMRig):
// piping their stdout/stderr/stdin, accessing their APIs (HTTP + disk files), etc.

//---------------------------------------------------------------------------------------------------- Import
use std::{
	sync::{Arc,Mutex},
	path::PathBuf,
	process::Command,
	time::*,
	thread,
};
use serde::{Serialize,Deserialize};
use crate::constants::*;
use log::*;


//---------------------------------------------------------------------------------------------------- [Helper] Struct
// A meta struct holding all the data that gets processed in this thread
pub struct Helper {
	pub instant: Instant,      // Gupax start as an [Instant]
	pub human_time: HumanTime, // Gupax uptime formatting for humans
	pub p2pool: Process,       // P2Pool process state
	pub xmrig: Process,        // XMRig process state
	pub pub_api_p2pool: PubP2poolApi, // P2Pool API state (for GUI thread)
	pub pub_api_xmrig: PubXmrigApi,   // XMRig API state (for GUI thread)
	priv_api_p2pool: PrivP2poolApi, // For "watchdog" thread
	priv_api_xmrig: PrivXmrigApi,   // For "watchdog" thread
}

// Impl found at the very bottom of this file.

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
pub struct Process {
	name: ProcessName,     // P2Pool or XMRig?
	state: ProcessState,   // The state of the process (alive, dead, etc)
	signal: ProcessSignal, // Did the user click [Start/Stop/Restart]?
	start: Instant,        // Start time of process
	uptime: HumanTime,     // Human readable process uptime
	output: String,        // This is the process's stdout + stderr
	stdin: Option<std::process::ChildStdin>, // A handle to the process's STDIN
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
	input: Vec<String>,
}

//---------------------------------------------------------------------------------------------------- [Process] Impl
impl Process {
	pub fn new(name: ProcessName, args: String, path: PathBuf) -> Self {
		let now = Instant::now();
		Self {
			name,
			state: ProcessState::Dead,
			signal: ProcessSignal::None,
			start: now,
			uptime: HumanTime::into_human(now.elapsed()),
			stdin: Option::None,
			// P2Pool log level 1 produces a bit less than 100,000 lines a day.
			// Assuming each line averages 80 UTF-8 scalars (80 bytes), then this
			// initial buffer should last around a week (56MB) before resetting.
			output: String::with_capacity(56_000_000),
			input: vec![String::new()],
		}
	}

	// Borrow a [&str], return an owned split collection
	pub fn parse_args(args: &str) -> Vec<String> {
		args.split_whitespace().map(|s| s.to_owned()).collect()
	}
}

//---------------------------------------------------------------------------------------------------- [Process*] Enum
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessState {
	Alive,    // Process is online, GREEN!
	Dead,     // Process is dead, BLACK!
	Failed,   // Process is dead AND exited with a bad code, RED!
	// Process is starting up, YELLOW!
	// Really, processes start instantly, this just accounts for the delay
	// between the main thread and this threads 1 second event loop.
	Starting,
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessSignal {
	None,
	Start,
	Stop,
	Restart,
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessName {
	P2pool,
	Xmrig,
}

impl std::fmt::Display for ProcessState  { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }
impl std::fmt::Display for ProcessSignal { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }
impl std::fmt::Display for ProcessName   { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }

//---------------------------------------------------------------------------------------------------- [HumanTime]
// This converts a [std::time::Duration] into something more readable.
// Used for uptime display purposes: [7 years, 8 months, 15 days, 23 hours, 35 minutes, 1 second]
// Code taken from [https://docs.rs/humantime/] and edited to remove sub-second time, change spacing and some words.
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HumanTime(Duration);

impl HumanTime {
	pub fn into_human(d: Duration) -> HumanTime {
		HumanTime(d)
	}

	fn plural(f: &mut std::fmt::Formatter, started: &mut bool, name: &str, value: u64) -> std::fmt::Result {
		if value > 0 {
			if *started { f.write_str(" ")?; }
		}
		write!(f, "{}{}", value, name)?;
		if value > 1 {
			f.write_str("s")?;
		}
		*started = true;
		Ok(())
	}
}

impl std::fmt::Display for HumanTime {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let secs = self.0.as_secs();
		if secs == 0 {
			f.write_str("0s")?;
			return Ok(());
		}

		let years = secs / 31_557_600;  // 365.25d
		let ydays = secs % 31_557_600;
		let months = ydays / 2_630_016;  // 30.44d
		let mdays = ydays % 2_630_016;
		let days = mdays / 86400;
		let day_secs = mdays % 86400;
		let hours = day_secs / 3600;
		let minutes = day_secs % 3600 / 60;
		let seconds = day_secs % 60;

		let ref mut started = false;
		Self::plural(f, started, " year", years)?;
		Self::plural(f, started, " month", months)?;
		Self::plural(f, started, " day", days)?;
		Self::plural(f, started, " hour", hours)?;
		Self::plural(f, started, " minute", minutes)?;
		Self::plural(f, started, " second", seconds)?;
		Ok(())
	}
}

//---------------------------------------------------------------------------------------------------- Public P2Pool API
// GUI thread interfaces with this.
pub struct PubP2poolApi {

}

impl PubP2poolApi {
	pub fn new() -> Self {
		Self {
		}
	}
}

//---------------------------------------------------------------------------------------------------- Private P2Pool API
// This is the data the "watchdog" threads mutate.
// It matches directly to P2Pool's [local/stats] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize)]
struct PrivP2poolApi {
	hashrate_15m: u128,
	hashrate_1h: u128,
	hashrate_24h: u128,
	shares_found: u128,
	average_effort: f64,
	current_effort: f64,
	connections: u16, // No one will have more than 65535 connections... right?
}

impl PrivP2poolApi {
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
}

//---------------------------------------------------------------------------------------------------- Public XMRig API
pub struct PubXmrigApi {

}

impl PubXmrigApi {
	pub fn new() -> Self {
		Self {
		}
	}
}

//---------------------------------------------------------------------------------------------------- Private XMRig API
// This matches to some JSON stats in the HTTP call [summary],
// e.g: [wget -qO- localhost:18085/1/summary].
// XMRig doesn't initialize stats at 0 (or 0.0) and instead opts for [null]
// which means some elements need to be wrapped in an [Option] or else serde will [panic!].
#[derive(Debug, Serialize, Deserialize)]
struct PrivXmrigApi {
	worker_id: String,
	resources: Resources,
	connection: Connection,
	hashrate: Hashrate,
}

impl PrivXmrigApi {
	fn new() -> Self {
		Self {
			worker_id: String::new(),
			resources: Resources::new(),
			connection: Connection::new(),
			hashrate: Hashrate::new(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct Resources {
	load_average: [Option<f32>; 3],
}
impl Resources {
	fn new() -> Self {
		Self {
			load_average: [Some(0.0), Some(0.0), Some(0.0)],
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct Connection {
	pool: String,
	ping: u32,
	diff: u128,
	accepted: u128,
	rejected: u128,
}
impl Connection {
	fn new() -> Self {
		Self {
			pool: String::new(),
			ping: 0,
			diff: 0,
			accepted: 0,
			rejected: 0,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct Hashrate {
	total: [Option<f32>; 3],
}
impl Hashrate {
	fn new() -> Self {
		Self {
			total: [Some(0.0), Some(0.0), Some(0.0)],
		}
	}
}

//---------------------------------------------------------------------------------------------------- [Helper]
use tokio::io::{BufReader,AsyncBufReadExt};

impl Helper {
	//---------------------------------------------------------------------------------------------------- General Functions
	pub fn new(instant: std::time::Instant) -> Self {
		Self {
			instant,
			human_time: HumanTime::into_human(instant.elapsed()),
			p2pool: Process::new(ProcessName::P2pool, String::new(), PathBuf::new()),
			xmrig: Process::new(ProcessName::Xmrig, String::new(), PathBuf::new()),
			pub_api_p2pool: PubP2poolApi::new(),
			pub_api_xmrig: PubXmrigApi::new(),
			priv_api_p2pool: PrivP2poolApi::new(),
			priv_api_xmrig: PrivXmrigApi::new(),
		}
	}

	// The tokio runtime that blocks while async reading both STDOUT/STDERR
	// Cheaper than spawning 2 OS threads just to read 2 pipes (...right? :D)
	#[tokio::main]
	async fn read_stdout_stderr(stdout: tokio::process::ChildStdout, stderr: tokio::process::ChildStderr) {
		// Create STDOUT pipe job
		let stdout_job = tokio::spawn(async move {
			let mut stdout_reader = BufReader::new(stdout).lines();
			while let Ok(Some(line)) = stdout_reader.next_line().await {
				println!("{}", line);
			}
		});
		// Create STDERR pipe job
		let stderr_job = tokio::spawn(async move {
			let mut stderr_reader = BufReader::new(stderr).lines();
			while let Ok(Some(line)) = stderr_reader.next_line().await {
			println!("{}", line);
			}
		});
		// Block and read both until they are closed (automatic when process dies)
		tokio::join![stdout_job, stderr_job];
	}

	//---------------------------------------------------------------------------------------------------- P2Pool specific
	// Intermediate function that parses the arguments, and spawns the P2Pool watchdog thread.
	pub fn spawn_p2pool(state: &crate::disk::P2pool, api_path: &std::path::Path) {
		let mut args = Vec::with_capacity(500);
		// [Simple]
		if state.simple {
			// Build the p2pool argument
			let (ip, rpc, zmq) = crate::node::enum_to_ip_rpc_zmq_tuple(state.node); // Get: (IP, RPC, ZMQ)
			args.push(format!("--wallet {}", state.address));        // Wallet Address
			args.push(format!("--host {}", ip));                     // IP Address
			args.push(format!("--rpc-port {}", rpc));                // RPC Port
			args.push(format!("--zmq-port {}", zmq));                // ZMQ Port
			args.push(format!("--data-api {}", api_path.display())); // API Path
			args.push("--local-api".to_string());                    // Enable API
			args.push("--no-color".to_string());                     // Remove color escape sequences, Gupax terminal can't parse it :(
			args.push("--mini".to_string());                         // P2Pool Mini

		// [Advanced]
		} else {
			// Overriding command arguments
			if !state.arguments.is_empty() {
				for arg in state.arguments.split_whitespace() {
					args.push(arg.to_string());
				}
			// Else, build the argument
			} else {
				args.push(state.address.clone());      // Wallet
				args.push(state.selected_ip.clone());  // IP
				args.push(state.selected_rpc.clone()); // RPC
				args.push(state.selected_zmq.clone()); // ZMQ
				args.push("--local-api".to_string());  // Enable API
				args.push("--no-color".to_string());   // Remove color escape sequences
				if state.mini { args.push("--mini".to_string()); };      // Mini
				args.push(format!("--loglevel {}", state.log_level));    // Log Level
				args.push(format!("--out-peers {}", state.out_peers));   // Out Peers
				args.push(format!("--in-peers {}", state.in_peers));     // In Peers
				args.push(format!("--data-api {}", api_path.display())); // API Path
			}
		}

		// Print arguments to console
		crate::disk::print_dash(&format!("P2Pool | Launch arguments ... {:#?}", args));

		// Spawn watchdog thread
		thread::spawn(move || {
			Self::spawn_p2pool_watchdog(args);
		});
	}

	// The actual P2Pool watchdog tokio runtime.
	#[tokio::main]
	pub async fn spawn_p2pool_watchdog(args: Vec<String>) {
		// 1. Create command
		// 2. Spawn STDOUT/STDERR thread
		// 3. Loop forever as watchdog until process dies
	}

	//---------------------------------------------------------------------------------------------------- XMRig specific
	// Intermediate function that parses the arguments, and spawns the XMRig watchdog thread.
	pub fn spawn_xmrig(state: &crate::disk::Xmrig, api_path: &std::path::Path) {
		let mut args = Vec::with_capacity(500);
		if state.simple {
			let rig_name = if state.simple_rig.is_empty() { GUPAX_VERSION.to_string() } else { state.simple_rig.clone() }; // Rig name
			args.push(format!("--threads {}", state.current_threads)); // Threads
			args.push(format!("--user {}", state.simple_rig));         // Rig name
			args.push(format!("--url 127.0.0.1:3333"));                // Local P2Pool (the default)
			args.push("--no-color".to_string());                       // No color escape codes
			if state.pause != 0 { args.push(format!("--pause-on-active {}", state.pause)); } // Pause on active
		} else {
			if !state.arguments.is_empty() {
				for arg in state.arguments.split_whitespace() {
					args.push(arg.to_string());
				}
			} else {
				args.push(format!("--user {}", state.address.clone()));    // Wallet
				args.push(format!("--threads {}", state.current_threads)); // Threads
				args.push(format!("--rig-id {}", state.selected_rig));     // Rig ID
				args.push(format!("--url {}:{}", state.selected_ip.clone(), state.selected_port.clone())); // IP/Port
				args.push(format!("--http-host {}", state.api_ip).to_string());   // HTTP API IP
				args.push(format!("--http-port {}", state.api_port).to_string()); // HTTP API Port
				args.push("--no-color".to_string());                         // No color escape codes
				if state.tls { args.push("--tls".to_string()); }             // TLS
				if state.keepalive { args.push("--keepalive".to_string()); } // Keepalive
				if state.pause != 0 { args.push(format!("--pause-on-active {}", state.pause)); } // Pause on active
			}
		}
		// Print arguments to console
		crate::disk::print_dash(&format!("XMRig | Launch arguments ... {:#?}", args));

		// Spawn watchdog thread
		thread::spawn(move || {
			Self::spawn_xmrig_watchdog(args);
		});
	}

	// The actual XMRig watchdog tokio runtime.
	#[tokio::main]
	pub async fn spawn_xmrig_watchdog(args: Vec<String>) {
	}

	//---------------------------------------------------------------------------------------------------- The "helper"
	// Intermediate function that spawns the helper thread.
	pub fn spawn_helper(helper: &Arc<Mutex<Self>>) {
		let helper = Arc::clone(helper);
		thread::spawn(move || { Self::helper(helper); });
	}

	// [helper] = Actual Arc
	// [h]      = Temporary lock that gets dropped
	// [jobs]   = Vector of async jobs ready to go
//	#[tokio::main]
	pub fn helper(helper: Arc<Mutex<Self>>) {
		// Begin loop
		loop {

		// 1. Create "jobs" vector holding async tasks
//		let jobs: Vec<tokio::task::JoinHandle<Result<(), anyhow::Error>>> = vec![];

		// 2. Loop init timestamp
		let start = Instant::now();

		// 3. Spawn child processes (if signal found)
		let h = helper.lock().unwrap();
		if let ProcessSignal::Start = h.p2pool.signal {
			// Start outer thread, start inner stdout/stderr pipe, loop in outer thread for stdin/signal/etc
			if !h.p2pool.input.is_empty() {
				// Process STDIN
			}
		}
		drop(h);
		let h = helper.lock().unwrap();
		if let ProcessSignal::Start = h.xmrig.signal {
			// Start outer thread, start inner stdout/stderr pipe, loop in outer thread for stdin/signal/etc
			if !h.xmrig.input.is_empty() {
				// Process STDIN
			}
		}
		drop(h);

		// 4. Collect P2Pool API task (if alive)
		let h = helper.lock().unwrap();
		if let ProcessState::Alive = h.p2pool.state {
		}
		// 5. Collect XMRig HTTP API task (if alive)
		if let ProcessState::Alive = h.xmrig.state {
		}
		drop(h);

		// 6. Execute all async tasks
//		for job in jobs {
//			job.await;
//		}

		// 7. Set Gupax/P2Pool/XMRig uptime
		let mut h = helper.lock().unwrap();
		h.human_time = HumanTime::into_human(h.instant.elapsed());
		drop(h);

		// 8. Calculate if we should sleep or not.
		// If we should sleep, how long?
		let elapsed = start.elapsed().as_millis();
		if elapsed < 1000 {
			// Casting from u128 to u64 should be safe here, because [elapsed]
			// is less than 1000, meaning it can fit into a u64 easy.
			std::thread::sleep(std::time::Duration::from_millis((1000-elapsed) as u64));
		}

		// 9. End loop
		}
	}
}
