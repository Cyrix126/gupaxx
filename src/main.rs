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

// Hide console in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//---------------------------------------------------------------------------------------------------- Imports
// egui/eframe
use egui::{
	TextStyle::*,
	color::Color32,
	FontFamily::Proportional,
	TextStyle,Spinner,
	Layout,Align,
	FontId,Label,RichText,Stroke,Vec2,Button,SelectableLabel,
	Key,Modifiers,TextEdit,
	CentralPanel,TopBottomPanel,
	Hyperlink,
};
use egui_extras::RetainedImage;
use eframe::{egui,NativeOptions};
// Logging
use log::*;
use env_logger::{Builder,WriteStyle};
// Regex
use regex::Regex;
// Serde
use serde::{Serialize,Deserialize};
// std
use std::{
	env,
	io::Write,
	process::exit,
	sync::{Arc,Mutex},
	time::Instant,
	path::PathBuf,
};
// Sysinfo
use sysinfo::SystemExt;
// Modules
mod ferris;
mod constants;
mod node;
mod disk;
mod status;
mod gupax;
mod p2pool;
mod xmrig;
mod update;
mod helper;
use {ferris::*,constants::*,node::*,disk::*,status::*,update::*,gupax::*,helper::*};

// Sudo (dummy values for Windows)
mod sudo;
use crate::sudo::*;
#[cfg(target_family = "unix")]
extern crate sudo as sudo_check;

//---------------------------------------------------------------------------------------------------- Struct + Impl
// The state of the outer main [App].
// See the [State] struct in [state.rs] for the
// actual inner state of the tab settings.
pub struct App {
	// Misc state
	tab: Tab, // What tab are we on?
	width: f32, // Top-level width
	height: f32, // Top-level height
	// Alpha (transparency)
	// This value is used to incrementally increase/decrease
	// the transparency when resizing. Basically, it fades
	// in/out of black to hide jitter when resizing with [init_text_styles()]
	alpha: u8,
	// This is a one time trigger so [init_text_styles()] isn't
	// called 60x a second when resizing the window. Instead,
	// it only gets called if this bool is true and the user
	// is hovering over egui (ctx.is_pointer_over_area()).
	must_resize: bool, // Sets the flag so we know to [init_text_styles()]
	resizing: bool,    // Are we in the process of resizing? (For black fade in/out)
	// State
	og: Arc<Mutex<State>>, // og = Old state to compare against
	state: State, // state = Working state (current settings)
	update: Arc<Mutex<Update>>, // State for update data [update.rs]
	file_window: Arc<Mutex<FileWindow>>, // State for the path selector in [Gupax]
	ping: Arc<Mutex<Ping>>, // Ping data found in [node.rs]
	og_node_vec: Vec<(String, Node)>, // Manual Node database
	node_vec: Vec<(String, Node)>, // Manual Node database
	og_pool_vec: Vec<(String, Pool)>, // Manual Pool database
	pool_vec: Vec<(String, Pool)>, // Manual Pool database
	diff: bool, // This bool indicates state changes
	// Restart state:
	// If Gupax updated itself, this represents that the
	// user should (but isn't required to) restart Gupax.
	restart: Arc<Mutex<Restart>>,
	// Error State:
	// These values are essentially global variables that
	// indicate if an error message needs to be displayed
	// (it takes up the whole screen with [error_msg] and buttons for ok/quit/etc)
	error_state: ErrorState,
	// Helper/API State:
	// This holds everything related to the data processed by the "helper thread".
	// This includes the "helper" threads public P2Pool/XMRig's API.
	helper: Arc<Mutex<Helper>>,  // [Helper] state, mostly for Gupax uptime
	pub_sys: Arc<Mutex<Sys>>, // [Sys] state, read by [Status], mutated by [Helper]
	p2pool: Arc<Mutex<Process>>, // [P2Pool] process state
	xmrig: Arc<Mutex<Process>>,  // [XMRig] process state
	p2pool_api: Arc<Mutex<PubP2poolApi>>, // Public ready-to-print P2Pool API made by the "helper" thread
	xmrig_api: Arc<Mutex<PubXmrigApi>>,   // Public ready-to-print XMRig API made by the "helper" thread
	p2pool_img: Arc<Mutex<ImgP2pool>>,    // A one-time snapshot of what data P2Pool started with
	xmrig_img: Arc<Mutex<ImgXmrig>>,      // A one-time snapshot of what data XMRig started with
	// Buffer State
	p2pool_console: String, // The buffer between the p2pool console and the [Helper]
	xmrig_console: String, // The buffer between the xmrig console and the [Helper]
	// Sudo State
	sudo: Arc<Mutex<SudoState>>, // This is just a dummy struct on [Windows].
	// State from [--flags]
	no_startup: bool,
	// Static stuff
	pid: sysinfo::Pid, // Gupax's PID
	max_threads: usize, // Max amount of detected system threads
	now: Instant, // Internal timer
	exe: String, // Path for [Gupax] binary
	dir: String, // Directory [Gupax] binary is in
	resolution: Vec2, // Frame resolution
	os: &'static str, // OS
	admin: bool, // Are we admin? (for Windows)
	os_data_path: PathBuf, // OS data path (e.g: ~/.local/share/gupax)
	state_path: PathBuf, // State file path
	node_path: PathBuf, // Node file path
	pool_path: PathBuf, // Pool file path
	version: &'static str, // Gupax version
	name_version: String, // [Gupax vX.X.X]
	img: Images, // Custom Struct holding pre-compiled bytes of [Images]
	regex: Regexes, // Custom Struct holding pre-made [Regex]'s
}

impl App {
	fn cc(cc: &eframe::CreationContext<'_>, app: Self) -> Self {
		let resolution = cc.integration_info.window_info.size;
		init_text_styles(&cc.egui_ctx, resolution[0]);
		Self {
			resolution,
			..app
		}
	}

	fn new(now: Instant) -> Self {
		info!("Initializing App Struct...");
		debug!("App Init | P2Pool & XMRig processes...");
		let p2pool = Arc::new(Mutex::new(Process::new(ProcessName::P2pool, String::new(), PathBuf::new())));
		let xmrig = Arc::new(Mutex::new(Process::new(ProcessName::Xmrig, String::new(), PathBuf::new())));
		let p2pool_api = Arc::new(Mutex::new(PubP2poolApi::new()));
		let xmrig_api = Arc::new(Mutex::new(PubXmrigApi::new()));
		let p2pool_img = Arc::new(Mutex::new(ImgP2pool::new()));
		let xmrig_img = Arc::new(Mutex::new(ImgXmrig::new()));

		debug!("App Init | Sysinfo...");
		// We give this to the [Helper] thread.
		let mut sysinfo = sysinfo::System::new_with_specifics(
			sysinfo::RefreshKind::new()
			.with_cpu(sysinfo::CpuRefreshKind::everything())
			.with_processes(sysinfo::ProcessRefreshKind::new().with_cpu())
			.with_memory());
		sysinfo.refresh_all();
		let pid = sysinfo::get_current_pid().unwrap();
		let pub_sys = Arc::new(Mutex::new(Sys::new()));

		debug!("App Init | The rest of the [App]...");
		let mut app = Self {
			tab: Tab::default(),
			ping: Arc::new(Mutex::new(Ping::new())),
			width: APP_DEFAULT_WIDTH,
			height: APP_DEFAULT_HEIGHT,
			must_resize: false,
			og: Arc::new(Mutex::new(State::new())),
			state: State::new(),
			update: Arc::new(Mutex::new(Update::new(String::new(), PathBuf::new(), PathBuf::new(), true))),
			file_window: FileWindow::new(),
			og_node_vec: Node::new_vec(),
			node_vec: Node::new_vec(),
			og_pool_vec: Pool::new_vec(),
			pool_vec: Pool::new_vec(),
			restart: Arc::new(Mutex::new(Restart::No)),
			diff: false,
			error_state: ErrorState::new(),
			helper: Arc::new(Mutex::new(Helper::new(now, pub_sys.clone(), p2pool.clone(), xmrig.clone(), p2pool_api.clone(), xmrig_api.clone(), p2pool_img.clone(), xmrig_img.clone()))),
			p2pool,
			xmrig,
			p2pool_api,
			xmrig_api,
			p2pool_img,
			xmrig_img,
			p2pool_console: String::with_capacity(10),
			xmrig_console: String::with_capacity(10),
			sudo: Arc::new(Mutex::new(SudoState::new())),
			resizing: false,
			alpha: 0,
			no_startup: false,
			pub_sys,
			pid,
			max_threads: num_cpus::get(),
			now,
			admin: false,
			exe: String::new(),
			dir: String::new(),
			resolution: Vec2::new(APP_DEFAULT_HEIGHT, APP_DEFAULT_WIDTH),
			os: OS,
			os_data_path: PathBuf::new(),
			state_path: PathBuf::new(),
			node_path: PathBuf::new(),
			pool_path: PathBuf::new(),
			version: GUPAX_VERSION,
			name_version: format!("Gupax {}", GUPAX_VERSION),
			img: Images::new(),
			regex: Regexes::new(),
		};
		//---------------------------------------------------------------------------------------------------- App init data that *could* panic
		debug!("App Init | Getting EXE path...");
		let mut panic = String::new();
		// Get exe path
		app.exe = match get_exe() {
			Ok(exe) => exe,
			Err(e) => { panic = format!("get_exe(): {}", e); app.error_state.set(panic.clone(), ErrorFerris::Panic, ErrorButtons::Quit); String::new() },
		};
		// Get exe directory path
		app.dir = match get_exe_dir() {
			Ok(dir) => dir,
			Err(e) => { panic = format!("get_exe_dir(): {}", e); app.error_state.set(panic.clone(), ErrorFerris::Panic, ErrorButtons::Quit); String::new() },
		};
		// Get OS data path
		app.os_data_path = match get_gupax_data_path() {
			Ok(dir) => dir,
			Err(e) => { panic = format!("get_os_data_path(): {}", e); app.error_state.set(panic.clone(), ErrorFerris::Panic, ErrorButtons::Quit); PathBuf::new() },
		};

		debug!("App Init | Setting TOML path...");
		// Set [*.toml] path
		app.state_path = app.os_data_path.clone();
		app.state_path.push("state.toml");
		app.node_path = app.os_data_path.clone();
		app.node_path.push("node.toml");
		app.pool_path = app.os_data_path.clone();
		app.pool_path.push("pool.toml");

		// Apply arg state
		// It's not safe to [--reset] if any of the previous variables
		// are unset (null path), so make sure we just abort if the [panic] String contains something.
		debug!("App Init | Applying argument state...");
		let mut app = parse_args(app, panic);

		// Read disk state
		debug!("App Init | Reading disk state...");
		use TomlError::*;
		app.state = match State::get(&app.state_path) {
			Ok(toml) => toml,
			Err(err) => {
				error!("State ... {}", err);
				match err {
					Io(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Path(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Serialize(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Deserialize(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Format(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Merge(e) => app.error_state.set(format!("State file: {}", e), ErrorFerris::Error, ErrorButtons::ResetState),
				};
				State::new()
			},
		};
		app.og = Arc::new(Mutex::new(app.state.clone()));
		// Read node list
		debug!("App Init | Reading node list...");
		app.node_vec = match Node::get(&app.node_path) {
			Ok(toml) => toml,
			Err(err) => {
				error!("Node ... {}", err);
				match err {
					Io(e) => app.error_state.set(format!("Node list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Path(e) => app.error_state.set(format!("Node list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Serialize(e) => app.error_state.set(format!("Node list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Deserialize(e) => app.error_state.set(format!("Node list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Format(e) => app.error_state.set(format!("Node file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Merge(e) => app.error_state.set(format!("Node list: {}", e), ErrorFerris::Error, ErrorButtons::ResetState),
				};
				Node::new_vec()
			},
		};
		app.og_node_vec = app.node_vec.clone();
		debug!("Node Vec:");
		debug!("{:#?}", app.node_vec);
		// Read pool list
		debug!("App Init | Reading pool list...");
		app.pool_vec = match Pool::get(&app.pool_path) {
			Ok(toml) => toml,
			Err(err) => {
				error!("Pool ... {}", err);
				match err {
					Io(e) => app.error_state.set(format!("Pool list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Path(e) => app.error_state.set(format!("Pool list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Serialize(e) => app.error_state.set(format!("Pool list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Deserialize(e) => app.error_state.set(format!("Pool list: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Format(e) => app.error_state.set(format!("Pool file: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
					Merge(e) => app.error_state.set(format!("Pool list: {}", e), ErrorFerris::Error, ErrorButtons::ResetState),
				};
				Pool::new_vec()
			},
		};
		app.og_pool_vec = app.pool_vec.clone();
		debug!("Pool Vec:");
		debug!("{:#?}", app.pool_vec);


		//----------------------------------------------------------------------------------------------------
		let mut og = app.og.lock().unwrap(); // Lock [og]
		// Handle max threads
		debug!("App Init | Handling max thread overflow...");
		og.xmrig.max_threads = app.max_threads;
		let current = og.xmrig.current_threads;
		let max = og.xmrig.max_threads;
		if current > max {
			og.xmrig.current_threads = max;
		}
		// Handle [node_vec] overflow
		debug!("App Init | Handling [node_vec] overflow");
		if og.p2pool.selected_index > app.og_node_vec.len() {
			warn!("App | Overflowing manual node index [{} > {}], resetting to 1", og.p2pool.selected_index, app.og_node_vec.len());
			let (name, node) = app.og_node_vec[0].clone();
			og.p2pool.selected_index = 0;
			og.p2pool.selected_name = name.clone();
			og.p2pool.selected_ip = node.ip.clone();
			og.p2pool.selected_rpc = node.rpc.clone();
			og.p2pool.selected_zmq = node.zmq.clone();
			app.state.p2pool.selected_index = 0;
			app.state.p2pool.selected_name = name;
			app.state.p2pool.selected_ip = node.ip;
			app.state.p2pool.selected_rpc = node.rpc;
			app.state.p2pool.selected_zmq = node.zmq;
		}
		// Handle [pool_vec] overflow
		debug!("App Init | Handling [pool_vec] overflow...");
		if og.xmrig.selected_index > app.og_pool_vec.len() {
			warn!("App | Overflowing manual pool index [{} > {}], resetting to 1", og.xmrig.selected_index, app.og_pool_vec.len());
			let (name, pool) = app.og_pool_vec[0].clone();
			og.xmrig.selected_index = 0;
			og.xmrig.selected_name = name.clone();
			og.xmrig.selected_ip = pool.ip.clone();
			og.xmrig.selected_port = pool.port.clone();
			app.state.xmrig.selected_index = 0;
			app.state.xmrig.selected_name = name;
			app.state.xmrig.selected_ip = pool.ip;
			app.state.xmrig.selected_port = pool.port;
		}

		// Apply TOML values to [Update]
		debug!("App Init | Applying TOML values to [Update]...");
		let p2pool_path = og.gupax.absolute_p2pool_path.clone();
		let xmrig_path = og.gupax.absolute_xmrig_path.clone();
		let tor = og.gupax.update_via_tor;
		app.update = Arc::new(Mutex::new(Update::new(app.exe.clone(), p2pool_path, xmrig_path, tor)));


		debug!("App Init | Setting state Gupax version...");
		// Set state version as compiled in version
		og.version.lock().unwrap().gupax = GUPAX_VERSION.to_string();
		app.state.version.lock().unwrap().gupax = GUPAX_VERSION.to_string();

		debug!("App Init | Setting saved [Tab]...");
		// Set saved [Tab]
		app.tab = app.state.gupax.tab;

		drop(og); // Unlock [og]
		info!("App ... OK");

		// Spawn the "Helper" thread.
		info!("Helper | Spawning helper thread...");
		Helper::spawn_helper(&app.helper, sysinfo, app.pid, app.max_threads);
		info!("Helper ... OK");

		// Check for privilege. Should be Admin on [Windows] and NOT root on Unix.
		debug!("App Init | Checking for privilege level...");
		#[cfg(target_os = "windows")]
		if is_elevated::is_elevated() {
			app.admin = true;
		} else {
			error!("Windows | Admin user not detected!");
			app.error_state.set(format!("Gupax was not launched as Administrator!\nBe warned, XMRig might have less hashrate!"), ErrorFerris::Sudo, ErrorButtons::WindowsAdmin);
		}
		#[cfg(target_family = "unix")]
		if sudo_check::check() != sudo_check::RunningAs::User {
			let id = sudo_check::check();
			error!("Unix | Regular user not detected: [{:?}]", id);
			app.error_state.set(format!("Gupax was launched as: [{:?}]\nPlease launch Gupax with regular user permissions.", id), ErrorFerris::Panic, ErrorButtons::Quit);
		}

		app
	}
}

//---------------------------------------------------------------------------------------------------- [Tab] Enum + Impl
// The tabs inside [App].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Tab {
	About,
	Status,
	Gupax,
	P2pool,
	Xmrig,
}

impl Default for Tab {
    fn default() -> Self {
        Self::About
    }
}

//---------------------------------------------------------------------------------------------------- [Restart] Enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restart {
	No, // We don't need to restart
	Yes, // We updated, user should probably (but isn't required to) restart
}

//---------------------------------------------------------------------------------------------------- [ErrorState] struct
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorButtons {
	YesNo,
	StayQuit,
	ResetState,
	ResetNode,
	Okay,
	Quit,
	Sudo,
	WindowsAdmin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorFerris {
	Happy,
	Oops,
	Error,
	Panic,
	Sudo,
}

pub struct ErrorState {
	error: bool, // Is there an error?
	msg: String, // What message to display?
	ferris: ErrorFerris, // Which ferris to display?
	buttons: ErrorButtons, // Which buttons to display?
}

impl Default for ErrorState {
	fn default() -> Self {
		Self::new()
	}
}

impl ErrorState {
	pub fn new() -> Self {
		Self {
			error: false,
			msg: "Unknown Error".to_string(),
			ferris: ErrorFerris::Oops,
			buttons: ErrorButtons::Okay,
		}
	}

	// Convenience function to enable the [App] error state
	pub fn set(&mut self, msg: impl Into<String>, ferris: ErrorFerris, buttons: ErrorButtons) {
		if self.error {
			// If a panic error is already set, return
			if self.ferris == ErrorFerris::Panic { return }
			// If we shouldn't be overriding the current error, return
			match self.buttons {
				ErrorButtons::YesNo => (), // Not important
				ErrorButtons::Okay => (), // Not important
				_ => return, // Overwrite, Quits, etc
			}
		}
		*self = Self {
			error: true,
			msg: msg.into(),
			ferris,
			buttons,
		};
	}

	// Just sets the current state to new, resetting it.
	pub fn reset(&mut self) {
		*self = Self::new();
	}

	// Instead of creating a whole new screen and system, this (ab)uses ErrorState
	// to ask for the [sudo] when starting XMRig. Yes, yes I know, it's called "ErrorState"
	// but rewriting the UI code and button stuff might be worse.
	// It also resets the current [SudoState]
	pub fn ask_sudo(&mut self, state: &Arc<Mutex<SudoState>>) {
		*self = Self {
			error: true,
			msg: String::new(),
			ferris: ErrorFerris::Sudo,
			buttons: ErrorButtons::Sudo,
		};
		SudoState::reset(state)
	}
}

//---------------------------------------------------------------------------------------------------- [Images] struct
struct Images {
	banner: RetainedImage,
	happy: RetainedImage,
	oops: RetainedImage,
	error: RetainedImage,
	panic: RetainedImage,
	sudo: RetainedImage,
}

impl Images {
	fn new() -> Self {
		Self {
			banner: RetainedImage::from_image_bytes("banner.png", BYTES_BANNER).unwrap(),
			happy: RetainedImage::from_image_bytes("happy.png", FERRIS_HAPPY).unwrap(),
			oops: RetainedImage::from_image_bytes("oops.png", FERRIS_OOPS).unwrap(),
			error: RetainedImage::from_image_bytes("error.png", FERRIS_ERROR).unwrap(),
			panic: RetainedImage::from_image_bytes("panic.png", FERRIS_PANIC).unwrap(),
			sudo: RetainedImage::from_image_bytes("panic.png", FERRIS_SUDO).unwrap(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- [Regexes] struct
#[derive(Clone, Debug)]
pub struct Regexes {
	pub name: Regex,
	pub address: Regex,
	pub ipv4: Regex,
	pub domain: Regex,
	pub port: Regex,
}

impl Regexes {
	fn new() -> Self {
		Regexes {
			name: Regex::new("^[A-Za-z0-9-_.]+( [A-Za-z0-9-_.]+)*$").unwrap(),
			address: Regex::new("^4[A-Za-z1-9]+$").unwrap(), // This still needs to check for (l, I, o, 0)
			ipv4: Regex::new(r#"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$"#).unwrap(),
			domain: Regex::new(r#"^(([a-zA-Z]{1})|([a-zA-Z]{1}[a-zA-Z]{1})|([a-zA-Z]{1}[0-9]{1})|([0-9]{1}[a-zA-Z]{1})|([a-zA-Z0-9][a-zA-Z0-9-_]{1,61}[a-zA-Z0-9]))\.([a-zA-Z]{2,6}|[a-zA-Z0-9-]{2,30}\.[a-zA-Z]{2,3})$"#).unwrap(),
			port: Regex::new(r#"^([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$"#).unwrap(),
		}
	}

	// Check if a Monero address is correct.
	pub fn addr_ok(&self, address: &str) -> bool {
		address.len() == 95 && Regex::is_match(&self.address, address) && !address.contains('0') && !address.contains('O') && !address.contains('l')
	}
}

//---------------------------------------------------------------------------------------------------- [Pressed] enum
// These represent the keys pressed during the frame.
// I could use egui's [Key] but there is no option for
// a [None] and wrapping [key_pressed] like [Option<egui::Key>]
// meant that I had to destructure like this:
//     if let Some(egui::Key)) = key_pressed { /* do thing */ }
//
// That's ugly, so these are used instead so a simple compare can be used.
#[derive(Debug,Clone,Eq,PartialEq)]
enum KeyPressed {
	F11,
	Up,
	Down,
	Esc,
	Z,
	X,
	S,
	R,
	None,
}

impl KeyPressed {
	fn is_f11(&self) -> bool {
		*self == Self::F11
	}
	fn is_z(&self) -> bool {
		*self == Self::Z
	}
	fn is_x(&self) -> bool {
		*self == Self::X
	}
	fn is_up(&self) -> bool {
		*self == Self::Up
	}
	fn is_down(&self) -> bool {
		*self == Self::Down
	}
	fn is_esc(&self) -> bool {
		*self == Self::Esc
	}
	fn is_s(&self) -> bool {
		*self == Self::S
	}
	fn is_r(&self) -> bool {
		*self == Self::R
	}
	fn is_none(&self) -> bool {
		*self == Self::None
	}
}

//---------------------------------------------------------------------------------------------------- Init functions
fn init_text_styles(ctx: &egui::Context, width: f32) {
	let scale = width / 30.0;
	let mut style = (*ctx.style()).clone();
	style.text_styles = [
		(Small, FontId::new(scale/3.0, Proportional)),
		(Body, FontId::new(scale/2.0, Proportional)),
		(Button, FontId::new(scale/2.0, Proportional)),
		(Monospace, FontId::new(scale/2.0, egui::FontFamily::Monospace)),
		(Heading, FontId::new(scale/1.5, Proportional)),
		(Name("Tab".into()), FontId::new(scale*1.2, Proportional)),
		(Name("Bottom".into()), FontId::new(scale/2.0, Proportional)),
		(Name("MonospaceSmall".into()), FontId::new(scale/2.5, egui::FontFamily::Monospace)),
		(Name("MonospaceLarge".into()), FontId::new(scale/1.5, egui::FontFamily::Monospace)),
	].into();
	style.spacing.icon_width_inner = width / 35.0;
	style.spacing.icon_width = width / 25.0;
	style.spacing.icon_spacing = 20.0;
	style.spacing.scroll_bar_width = width / 150.0;
	ctx.set_style(style);
	ctx.set_pixels_per_point(1.0);
	ctx.request_repaint();
}

fn init_logger(now: Instant) {
	use env_logger::fmt::Color;
	Builder::new().format(move |buf, record| {
		let mut style = buf.style();
		let level = match record.level() {
			Level::Error => { style.set_color(Color::Red); "ERROR" },
			Level::Warn => { style.set_color(Color::Yellow); "WARN" },
			Level::Info => { style.set_color(Color::White); "INFO" },
			Level::Debug => { style.set_color(Color::Blue); "DEBUG" },
			Level::Trace => { style.set_color(Color::Magenta); "TRACE" },
		};
		writeln!(
			buf,
			"[{}] [{}] [{}:{}] {}",
			style.set_bold(true).value(level),
			buf.style().set_dimmed(true).value(format!("{:.3}", now.elapsed().as_secs_f32())),
			buf.style().set_dimmed(true).value(record.file().unwrap_or("???")),
			buf.style().set_dimmed(true).value(record.line().unwrap_or(0)),
			record.args(),
		)
	}).filter_level(LevelFilter::Info).write_style(WriteStyle::Always).parse_default_env().format_timestamp_millis().init();
	info!("init_logger() ... OK");
}

fn init_options(initial_window_size: Option<Vec2>) -> NativeOptions {
	let mut options = eframe::NativeOptions::default();
	options.min_window_size = Some(Vec2::new(APP_MIN_WIDTH, APP_MIN_HEIGHT));
	options.max_window_size = Some(Vec2::new(APP_MAX_WIDTH, APP_MAX_HEIGHT));
	options.initial_window_size = initial_window_size;
	options.follow_system_theme = false;
	options.default_theme = eframe::Theme::Dark;
	let icon = image::load_from_memory(BYTES_ICON).expect("Failed to read icon bytes").to_rgba8();
	let (icon_width, icon_height) = icon.dimensions();
	options.icon_data = Some(eframe::IconData {
		rgba: icon.into_raw(),
		width: icon_width,
		height: icon_height,
	});
	info!("init_options() ... OK");
	options
}

fn init_auto(app: &mut App) {
	// Return early if [--no-startup] was not passed
	if app.no_startup {
		info!("[--no-startup] flag passed, skipping init_auto()...");
		return
	} else if app.error_state.error {
		info!("App error detected, skipping init_auto()...");
		return
	} else {
		info!("Starting init_auto()...");
	}

	// [Auto-Update]
	if app.state.gupax.auto_update {
		Update::spawn_thread(&app.og, &app.state.gupax, &app.state_path, &app.update, &mut app.error_state, &app.restart);
	} else {
		info!("Skipping auto-update...");
	}

	// [Auto-Ping]
	if app.state.p2pool.auto_ping && app.state.p2pool.simple {
		Ping::spawn_thread(&app.ping)
	} else {
		info!("Skipping auto-ping...");
	}

	// [Auto-P2Pool]
	if app.state.gupax.auto_p2pool {
		if !Regexes::addr_ok(&app.regex, &app.state.p2pool.address) {
			warn!("Gupax | P2Pool address is not valid! Skipping auto-p2pool...");
		} else if !Gupax::path_is_file(&app.state.gupax.p2pool_path) {
			warn!("Gupax | P2Pool path is not a file! Skipping auto-p2pool...");
		} else if !crate::update::check_p2pool_path(&app.state.gupax.p2pool_path) {
			warn!("Gupax | P2Pool path is not valid! Skipping auto-p2pool...");
		} else {
			Helper::start_p2pool(&app.helper, &app.state.p2pool, &app.state.gupax.absolute_p2pool_path);
		}
	} else {
		info!("Skipping auto-p2pool...");
	}

	// [Auto-XMRig]
	if app.state.gupax.auto_xmrig {
		if !Gupax::path_is_file(&app.state.gupax.xmrig_path) {
			warn!("Gupax | XMRig path is not an executable! Skipping auto-xmrig...");
		} else if !crate::update::check_xmrig_path(&app.state.gupax.xmrig_path) {
			warn!("Gupax | XMRig path is not valid! Skipping auto-xmrig...");
		} else if cfg!(windows) {
			Helper::start_xmrig(&app.helper, &app.state.xmrig, &app.state.gupax.absolute_xmrig_path, Arc::clone(&app.sudo));
		} else {
			app.sudo.lock().unwrap().signal = ProcessSignal::Start;
			app.error_state.ask_sudo(&app.sudo);
		}
	} else {
		info!("Skipping auto-xmrig...");
	}
}

//---------------------------------------------------------------------------------------------------- Reset functions
fn reset_state(path: &PathBuf) -> Result<(), TomlError> {
	match State::create_new(path) {
		Ok(_)  => { info!("Resetting [state.toml] ... OK"); Ok(()) },
		Err(e) => { error!("Resetting [state.toml] ... FAIL ... {}", e); Err(e) },
	}
}

fn reset_nodes(path: &PathBuf) -> Result<(), TomlError> {
	match Node::create_new(path) {
		Ok(_)  => { info!("Resetting [node.toml] ... OK"); Ok(()) },
		Err(e) => { error!("Resetting [node.toml] ... FAIL ... {}", e); Err(e) },
	}
}

fn reset_pools(path: &PathBuf) -> Result<(), TomlError> {
	match Pool::create_new(path) {
		Ok(_)  => { info!("Resetting [pool.toml] ... OK"); Ok(()) },
		Err(e) => { error!("Resetting [pool.toml] ... FAIL ... {}", e); Err(e) },
	}
}

fn reset(path: &PathBuf, state: &PathBuf, node: &PathBuf, pool: &PathBuf) {
	let mut code = 0;
	// Attempt to remove directory first
	match std::fs::remove_dir_all(path) {
		Ok(_) => info!("Removing OS data path ... OK"),
		Err(e) => { error!("Removing OS data path ... FAIL ... {}", e); code = 1; },
	}
	// Recreate
	match create_gupax_dir(path) {
		Ok(_) => (),
		Err(_) => code = 1,
	}
	match reset_state(state) {
		Ok(_) => (),
		Err(_) => code = 1,
	}
	match reset_nodes(node) {
		Ok(_) => (),
		Err(_) => code = 1,
	}
	match reset_pools(pool) {
		Ok(_) => (),
		Err(_) => code = 1,
	}
	match code {
		0 => println!("\nGupax reset ... OK"),
		_ => eprintln!("\nGupax reset ... FAIL"),
	}
	exit(code);
}

//---------------------------------------------------------------------------------------------------- Misc functions
fn parse_args<S: Into<String>>(mut app: App, panic: S) -> App {
	info!("Parsing CLI arguments...");
	let mut args: Vec<String> = env::args().collect();
	if args.len() == 1 { info!("No args ... OK"); return app } else { args.remove(0); info!("Args ... {:?}", args); }
	// [help/version], exit early
	for arg in &args {
		match arg.as_str() {
			"--help"    => { println!("{}", ARG_HELP); exit(0); },
			"--version" => {
				println!("Gupax {} [OS: {}, Commit: {}]\nThis Gupax was originally bundled with:\n    - P2Pool {}\n    - XMRig {}\n\n{}", GUPAX_VERSION, OS_NAME, &COMMIT[..40], P2POOL_VERSION, XMRIG_VERSION, ARG_COPYRIGHT);
				exit(0);
			},
			"--ferris" => { println!("{}", FERRIS_ANSI); exit(0); },
			_ => (),
		}
	}
	// Abort on panic
	let panic = panic.into();
	if ! panic.is_empty() {
		info!("[Gupax error] {}", panic);
		exit(1);
	}

	// Everything else
	for arg in args {
		match arg.as_str() {
			"--state"       => { info!("Printing state..."); print_disk_file(&app.state_path); }
			"--nodes"       => { info!("Printing node list..."); print_disk_file(&app.node_path); }
			"--reset-state" => if let Ok(()) = reset_state(&app.state_path) { println!("\nState reset ... OK"); exit(0); } else { eprintln!("\nState reset ... FAIL"); exit(1) },
			"--reset-nodes" => if let Ok(()) = reset_nodes(&app.node_path) { println!("\nNode reset ... OK"); exit(0) } else { eprintln!("\nNode reset ... FAIL"); exit(1) },
			"--reset-pools" => if let Ok(()) = reset_pools(&app.pool_path) { println!("\nPool reset ... OK"); exit(0) } else { eprintln!("\nPool reset ... FAIL"); exit(1) },
			"--reset-all"   => reset(&app.os_data_path, &app.state_path, &app.node_path, &app.pool_path),
			"--no-startup"  => app.no_startup = true,
			_               => { eprintln!("\n[Gupax error] Invalid option: [{}]\nFor help, use: [--help]", arg); exit(1); },
		}
	}
	app
}

// Get absolute [Gupax] binary path
pub fn get_exe() -> Result<String, std::io::Error> {
	match std::env::current_exe() {
		Ok(path) => { Ok(path.display().to_string()) },
		Err(err) => { error!("Couldn't get absolute Gupax PATH"); Err(err) },
	}
}

// Get absolute [Gupax] directory path
pub fn get_exe_dir() -> Result<String, std::io::Error> {
	match std::env::current_exe() {
		Ok(mut path) => { path.pop(); Ok(path.display().to_string()) },
		Err(err) => { error!("Couldn't get exe basepath PATH"); Err(err) },
	}
}

// Clean any [gupax_update_.*] directories
// The trailing random bits must be exactly 10 alphanumeric characters
pub fn clean_dir() -> Result<(), anyhow::Error> {
	let regex = Regex::new("^gupax_update_[A-Za-z0-9]{10}$").unwrap();
	for entry in std::fs::read_dir(get_exe_dir()?)? {
		let entry = entry?;
		if ! entry.path().is_dir() { continue }
		if Regex::is_match(&regex, entry.file_name().to_str().ok_or_else(|| anyhow::Error::msg("Basename failed"))?) {
			let path = entry.path();
			match std::fs::remove_dir_all(&path) {
				Ok(_) => info!("Remove [{}] ... OK", path.display()),
				Err(e) => warn!("Remove [{}] ... FAIL ... {}", path.display(), e),
			}
		}
	}
	Ok(())
}

// Print disk files to console
fn print_disk_file(path: &PathBuf) {
	match std::fs::read_to_string(path) {
		Ok(string) => { print!("{}", string); exit(0); },
		Err(e) => { error!("{}", e); exit(1); },
	}
}

//---------------------------------------------------------------------------------------------------- Main [App] frame
fn main() {
	let now = Instant::now();
	init_logger(now);
	let mut app = App::new(now);
	init_auto(&mut app);
	let selected_width = app.state.gupax.selected_width as f32;
	let selected_height = app.state.gupax.selected_height as f32;
	let initial_window_size = if selected_width > APP_MAX_WIDTH || selected_height > APP_MAX_HEIGHT {
		warn!("App | Set width or height was greater than the maximum! Starting with the default resolution...");
		Some(Vec2::new(APP_DEFAULT_WIDTH, APP_DEFAULT_HEIGHT))
	} else {
		Some(Vec2::new(app.state.gupax.selected_width as f32, app.state.gupax.selected_height as f32))
	};
	let options = init_options(initial_window_size);
	match clean_dir() {
		Ok(_) => info!("Temporary folder cleanup ... OK"),
		Err(e) => warn!("Could not cleanup [gupax_tmp] folders: {}", e),
	}
	info!("Init ... DONE");
	eframe::run_native(&app.name_version.clone(), options, Box::new(|cc| Box::new(App::cc(cc, app))),);
}

impl eframe::App for App {
	fn on_close_event(&mut self) -> bool {
		if self.state.gupax.ask_before_quit {
			self.error_state.set("", ErrorFerris::Oops, ErrorButtons::StayQuit);
			false
		} else {
			true
		}
	}

	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		// *-------*
		// | DEBUG |
		// *-------*
		debug!("App | ----------- Start of [update()] -----------");

		// If [F11] was pressed, reverse [fullscreen] bool
		let mut input = ctx.input_mut();
		let key: KeyPressed = {
			if input.consume_key(Modifiers::NONE, Key::F11) {
				KeyPressed::F11
			} else if input.consume_key(Modifiers::NONE, Key::Z) {
				KeyPressed::Z
			} else if input.consume_key(Modifiers::NONE, Key::X) {
				KeyPressed::X
			} else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
				KeyPressed::Up
			} else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
				KeyPressed::Down
			} else if input.consume_key(Modifiers::NONE, Key::Escape) {
				KeyPressed::Esc
			} else if input.consume_key(Modifiers::NONE, Key::S) {
				KeyPressed::S
			} else if input.consume_key(Modifiers::NONE, Key::R) {
				KeyPressed::R
			} else {
				KeyPressed::None
			}
		};
		drop(input);
		// Check if egui wants keyboard input.
		// This prevents keyboard shortcuts from clobbering TextEdits.
		// (Typing S in text would always [Save] instead)
		let wants_input = ctx.wants_keyboard_input();

        if key.is_f11() {
            let info = frame.info();
            frame.set_fullscreen(!info.window_info.fullscreen);
		// Change Tabs LEFT
        } else if key.is_z() && !wants_input {
			match self.tab {
				Tab::About  => self.tab = Tab::Xmrig,
				Tab::Status => self.tab = Tab::About,
				Tab::Gupax  => self.tab = Tab::Status,
				Tab::P2pool => self.tab = Tab::Gupax,
				Tab::Xmrig  => self.tab = Tab::P2pool,
			};
		// Change Tabs RIGHT
        } else if key.is_x() && !wants_input {
			match self.tab {
				Tab::About  => self.tab = Tab::Status,
				Tab::Status => self.tab = Tab::Gupax,
				Tab::Gupax  => self.tab = Tab::P2pool,
				Tab::P2pool => self.tab = Tab::Xmrig,
				Tab::Xmrig  => self.tab = Tab::About,
			};
        }

		// Refresh AT LEAST once a second
		debug!("App | Refreshing frame once per second");
		ctx.request_repaint_after(SECOND);

		// Get P2Pool/XMRig process state.
		// These values are checked multiple times so
		// might as well check only once here to save
		// on a bunch of [.lock().unwrap()]s.
		debug!("App | Locking and collecting P2Pool state...");
		let p2pool = self.p2pool.lock().unwrap();
		let p2pool_is_alive = p2pool.is_alive();
		let p2pool_is_waiting = p2pool.is_waiting();
		let p2pool_state = p2pool.state;
		drop(p2pool);
		debug!("App | Locking and collecting XMRig state...");
		let xmrig = self.xmrig.lock().unwrap();
		let xmrig_is_alive = xmrig.is_alive();
		let xmrig_is_waiting = xmrig.is_waiting();
		let xmrig_state = xmrig.state;
		drop(xmrig);

		// This sets the top level Ui dimensions.
		// Used as a reference for other uis.
		debug!("App | Setting width/height");
		CentralPanel::default().show(ctx, |ui| {
			let available_width = ui.available_width();
			if self.width != available_width {
				self.width = available_width;
				if self.now.elapsed().as_secs() > 5 {
					self.must_resize = true;
				}
			};
			self.height = ui.available_height();
		});
		// This resizes fonts/buttons/etc globally depending on the width.
		// This is separate from the [self.width != available_width] logic above
		// because placing [init_text_styles()] above would mean calling it 60x a second
		// while the user was readjusting the frame. It's a pretty heavy operation and looks
		// buggy when calling it that many times. Looking for a [must_resize] in addtion to
		// checking if the user is hovering over the app means that we only have call it once.
		debug!("App | Checking if we need to resize");
		if self.must_resize && ctx.is_pointer_over_area() {
			self.resizing = true;
			self.must_resize = false;
		}
		// This (ab)uses [Area] and [TextEdit] to overlay a full black layer over whatever UI we had before.
		// It incrementally becomes more opaque until [self.alpha] >= 250, when we just switch to pure black (no alpha).
		// When black, we're safe to [init_text_styles()], and then incrementally go transparent, until we remove the layer.
		if self.resizing {
			egui::Area::new("resize_layer").order(egui::Order::Foreground).anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0)).show(ctx, |ui| {
				if self.alpha < 250 {
					egui::Frame::none().fill(Color32::from_rgba_premultiplied(0,0,0,self.alpha)).show(ui, |ui| {
						ui.add_sized([ui.available_width()+SPACE, ui.available_height()+SPACE], egui::TextEdit::multiline(&mut ""));
					});
					ctx.request_repaint();
					self.alpha += 10;
				} else {
					egui::Frame::none().fill(Color32::from_rgb(0,0,0)).show(ui, |ui| {
						ui.add_sized([ui.available_width()+SPACE, ui.available_height()+SPACE], egui::TextEdit::multiline(&mut ""));
					});
					ctx.request_repaint();
					info!("App | Resizing frame to match new internal resolution: [{}x{}]", self.width, self.height);
					init_text_styles(ctx, self.width);
					self.resizing = false;
				}
			});
		} else if self.alpha != 0 {
			egui::Area::new("resize_layer").order(egui::Order::Foreground).anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0)).show(ctx, |ui| {
				egui::Frame::none().fill(Color32::from_rgba_premultiplied(0,0,0,self.alpha)).show(ui, |ui| {
					ui.add_sized([ui.available_width()+SPACE, ui.available_height()+SPACE], egui::TextEdit::multiline(&mut ""));
				})
			});
			self.alpha -= 10;
			ctx.request_repaint();
		}

		// If there's an error, display [ErrorState] on the whole screen until user responds
		debug!("App | Checking if there is an error in [ErrorState]");
		if self.error_state.error {
			CentralPanel::default().show(ctx, |ui| {
			ui.vertical_centered(|ui| {
				// Set width/height/font
				let width = self.width;
				let height = self.height/4.0;
				ui.style_mut().override_text_style = Some(Name("MonospaceLarge".into()));

				// Display ferris
				use ErrorFerris::*;
				use ErrorButtons::*;
				let ferris = match self.error_state.ferris {
					Happy => &self.img.happy,
					Oops  => &self.img.oops,
					Error => &self.img.error,
					Panic => &self.img.panic,
					ErrorFerris::Sudo => &self.img.sudo,
				};
				ferris.show_max_size(ui, Vec2::new(width, height));

				// Error/Quit screen
				match self.error_state.buttons {
					StayQuit => {
						let mut text = "".to_string();
						if *self.update.lock().unwrap().updating.lock().unwrap() { text = format!("{}\nUpdate is in progress...! Quitting may cause file corruption!", text); }
						if p2pool_is_alive { text = format!("{}\nP2Pool is online...!", text); }
						if xmrig_is_alive { text = format!("{}\nXMRig is online...!", text); }
						ui.add_sized([width, height], Label::new("--- Are you sure you want to quit? ---"));
						ui.add_sized([width, height], Label::new(text))
					},
					ResetState => {
						ui.add_sized([width, height], Label::new(format!("--- Gupax has encountered an error! ---\n{}", &self.error_state.msg)));
						ui.add_sized([width, height], Label::new("Reset Gupax state? (Your settings)"))
					},
					ResetNode  => {
						ui.add_sized([width, height], Label::new(format!("--- Gupax has encountered an error! ---\n{}", &self.error_state.msg)));
						ui.add_sized([width, height], Label::new("Reset the manual node list?"))
					},
					ErrorButtons::Sudo => {
						let text = format!("Why does XMRig need admin priviledge?\n{}", XMRIG_ADMIN_REASON);
						let height = height/4.0;
						ui.add_sized([width, height], Label::new(format!("--- Gupax needs sudo/admin priviledge for XMRig! ---\n{}", &self.error_state.msg)));
						ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
						ui.add_sized([width/2.0, height], Label::new(text));
						ui.add_sized([width, height], Hyperlink::from_label_and_url("Click here for more info.", "https://xmrig.com/docs/miner/randomx-optimization-guide"))
					},
					_ => {
						match self.error_state.ferris {
							Panic => ui.add_sized([width, height], Label::new("--- Gupax has encountered an un-recoverable error! ---")),
							Happy => ui.add_sized([width, height], Label::new("--- Success! ---")),
							_ => ui.add_sized([width, height], Label::new("--- Gupax has encountered an error! ---")),
						};
						let height = height/2.0;
						// Show GitHub rant link for Windows admin problems.
						if cfg!(windows) && self.error_state.buttons == ErrorButtons::WindowsAdmin {
							ui.add_sized([width, height], Hyperlink::from_label_and_url(
								"[Why does Gupax need to be Admin? (on Windows)]",
								"https://github.com/hinto-janaiyo/gupax/tree/main/src#why-does-gupax-need-to-be-admin-on-windows"
							));
							ui.add_sized([width, height], Label::new(&self.error_state.msg))
						} else {
							ui.add_sized([width, height], Label::new(&self.error_state.msg))
						}
					},
				};
				let height = ui.available_height();

				match self.error_state.buttons {
					YesNo   => {
						if ui.add_sized([width, height/2.0], Button::new("Yes")).clicked() { self.error_state.reset() }
						// If [Esc] was pressed, assume [No]
				        if key.is_esc() || ui.add_sized([width, height/2.0], Button::new("No")).clicked() { exit(0); }
					},
					StayQuit => {
						// If [Esc] was pressed, assume [Stay]
				        if key.is_esc() || ui.add_sized([width, height/2.0], Button::new("Stay")).clicked() {
							self.error_state = ErrorState::new();
						}
						if ui.add_sized([width, height/2.0], Button::new("Quit")).clicked() { exit(0); }
					},
					// This code handles the [state.toml/node.toml] resetting, [panic!]'ing if it errors once more
					// Another error after this either means an IO error or permission error, which Gupax can't fix.
					// [Yes/No] buttons
					ResetState => {
						if ui.add_sized([width, height/2.0], Button::new("Yes")).clicked() {
							match reset_state(&self.state_path) {
								Ok(_)  => {
									match State::get(&self.state_path) {
										Ok(s) => {
											self.state = s;
											self.og = Arc::new(Mutex::new(self.state.clone()));
											self.error_state.set("State read OK", ErrorFerris::Happy, ErrorButtons::Okay);
										},
										Err(e) => self.error_state.set(format!("State read fail: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
									}
								},
								Err(e) => self.error_state.set(format!("State reset fail: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
							};
						}
				        if key.is_esc() || ui.add_sized([width, height/2.0], Button::new("No")).clicked() { self.error_state.reset() }
					},
					ResetNode => {
						if ui.add_sized([width, height/2.0], Button::new("Yes")).clicked() {
							match reset_nodes(&self.node_path) {
								Ok(_)  => {
									match Node::get(&self.node_path) {
										Ok(s) => {
											self.node_vec = s;
											self.og_node_vec = self.node_vec.clone();
											self.error_state.set("Node read OK", ErrorFerris::Happy, ErrorButtons::Okay);
										},
										Err(e) => self.error_state.set(format!("Node read fail: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
									}
								},
								Err(e) => self.error_state.set(format!("Node reset fail: {}", e), ErrorFerris::Panic, ErrorButtons::Quit),
							};
						}
				        if key.is_esc() || ui.add_sized([width, height/2.0], Button::new("No")).clicked() { self.error_state.reset() }
					},
					ErrorButtons::Sudo => {
						let sudo_width = width/10.0;
						let height = ui.available_height()/4.0;
						let mut sudo = self.sudo.lock().unwrap();
						let hide = sudo.hide;
						ui.style_mut().override_text_style = Some(Monospace);
						if sudo.testing {
							ui.add_sized([width, height], Spinner::new().size(height));
							ui.set_enabled(false);
						} else {
							ui.add_sized([width, height], Label::new(&sudo.msg));
						}
						ui.add_space(height);
						let height = ui.available_height()/5.0;
						// Password input box with a hider.
						ui.horizontal(|ui| {
							let response = ui.add_sized([sudo_width*8.0, height], TextEdit::hint_text(TextEdit::singleline(&mut sudo.pass).password(hide), PASSWORD_TEXT));
							let box_width = (ui.available_width()/2.0)-5.0;
							if (response.lost_focus() && ui.input().key_pressed(Key::Enter)) ||
							ui.add_sized([box_width, height], Button::new("Enter")).on_hover_text(PASSWORD_ENTER).clicked() {
								response.request_focus();
								if !sudo.testing {
									SudoState::test_sudo(self.sudo.clone(), &self.helper.clone(), &self.state.xmrig, &self.state.gupax.absolute_xmrig_path);
								}
							}
							let color = if hide { BLACK } else { BRIGHT_YELLOW };
							if ui.add_sized([box_width, height], Button::new(RichText::new("👁").color(color))).on_hover_text(PASSWORD_HIDE).clicked() { sudo.hide = !sudo.hide; }
						});
						if (key.is_esc() && !sudo.testing) || ui.add_sized([width, height*4.0], Button::new("Leave")).clicked() { self.error_state.reset(); };
						// If [test_sudo()] finished, reset error state.
						if sudo.success {
							self.error_state.reset();
						}
					},
					Okay|WindowsAdmin => if key.is_esc() || ui.add_sized([width, height], Button::new("Okay")).clicked() { self.error_state.reset(); },
					Quit => if ui.add_sized([width, height], Button::new("Quit")).clicked() { exit(1); },
				}
			})});
			return
		}

		// Compare [og == state] & [node_vec/pool_vec] and enable diff if found.
		// The struct fields are compared directly because [Version]
		// contains Arc<Mutex>'s that cannot be compared easily.
		// They don't need to be compared anyway.
		debug!("App | Checking diff between [og] & [state]");
		let og = self.og.lock().unwrap();
		if og.gupax != self.state.gupax       ||
			og.p2pool != self.state.p2pool    ||
			og.xmrig != self.state.xmrig      ||
			self.og_node_vec != self.node_vec ||
			self.og_pool_vec != self.pool_vec {
			self.diff = true;
		} else {
			self.diff = false;
		}
		drop(og);

		// Top: Tabs
		debug!("App | Rendering TOP tabs");
		TopBottomPanel::top("top").show(ctx, |ui| {
			let width = (self.width - (SPACE*10.0))/5.0;
			let height = self.height/12.0;
		    ui.add_space(4.0);
			ui.horizontal(|ui| {
				let style = ui.style_mut();
				style.override_text_style = Some(Name("Tab".into()));
				style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(100, 100, 100);
				style.visuals.selection.bg_fill = Color32::from_rgb(255, 120, 120);
				style.visuals.selection.stroke = Stroke { width: 5.0, color: Color32::from_rgb(255, 255, 255) };
				if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::About, "About")).clicked() { self.tab = Tab::About; }
				ui.separator();
				if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Status, "Status")).clicked() { self.tab = Tab::Status; }
				ui.separator();
				if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Gupax, "Gupax")).clicked() { self.tab = Tab::Gupax; }
				ui.separator();
				if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::P2pool, "P2Pool")).clicked() { self.tab = Tab::P2pool; }
				ui.separator();
				if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Xmrig, "XMRig")).clicked() { self.tab = Tab::Xmrig; }
			});
			ui.add_space(4.0);
		});

		// Bottom: app info + state/process buttons
		debug!("App | Rendering BOTTOM bar");
		TopBottomPanel::bottom("bottom").show(ctx, |ui| {
			let height = self.height/20.0;
			ui.style_mut().override_text_style = Some(Name("Bottom".into()));
			ui.horizontal(|ui| {
				ui.group(|ui| {
					let width = ((self.width/2.0)/4.0)-(SPACE*2.0);
					// [Gupax Version]
					// Is yellow if the user updated and should (but isn't required to) restart.
					match *self.restart.lock().unwrap() {
						Restart::Yes => ui.add_sized([width, height], Label::new(RichText::new(&self.name_version).color(YELLOW))).on_hover_text(GUPAX_SHOULD_RESTART),
						_ => ui.add_sized([width, height], Label::new(&self.name_version)).on_hover_text(GUPAX_UP_TO_DATE),
					};
					ui.separator();
					// [OS]
					// Check if admin for windows.
					// Unix SHOULDN'T be running as root, and the check is done when
					// [App] is initialized, so no reason to check here.
					#[cfg(target_os = "windows")]
					if self.admin {
						ui.add_sized([width, height], Label::new(self.os));
					} else {
						ui.add_sized([width, height], Label::new(RichText::new(self.os).color(RED))).on_hover_text(WINDOWS_NOT_ADMIN);
					}
					#[cfg(target_family = "unix")]
					ui.add_sized([width, height], Label::new(self.os));
					ui.separator();
					// [P2Pool/XMRig] Status
					use ProcessState::*;
					match p2pool_state {
						Alive  => ui.add_sized([width, height], Label::new(RichText::new("P2Pool  ⏺").color(GREEN))).on_hover_text(P2POOL_ALIVE),
						Dead   => ui.add_sized([width, height], Label::new(RichText::new("P2Pool  ⏺").color(GRAY))).on_hover_text(P2POOL_DEAD),
						Failed => ui.add_sized([width, height], Label::new(RichText::new("P2Pool  ⏺").color(RED))).on_hover_text(P2POOL_FAILED),
						Middle|Waiting => ui.add_sized([width, height], Label::new(RichText::new("P2Pool  ⏺").color(YELLOW))).on_hover_text(P2POOL_MIDDLE),
					};
					ui.separator();
					match xmrig_state {
						Alive  => ui.add_sized([width, height], Label::new(RichText::new("XMRig  ⏺").color(GREEN))).on_hover_text(XMRIG_ALIVE),
						Dead   => ui.add_sized([width, height], Label::new(RichText::new("XMRig  ⏺").color(GRAY))).on_hover_text(XMRIG_DEAD),
						Failed => ui.add_sized([width, height], Label::new(RichText::new("XMRig  ⏺").color(RED))).on_hover_text(XMRIG_FAILED),
						Middle|Waiting => ui.add_sized([width, height], Label::new(RichText::new("XMRig  ⏺").color(YELLOW))).on_hover_text(XMRIG_MIDDLE),
					};
				});

				// [Save/Reset]
				ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
				let width = match self.tab {
					Tab::Gupax => (ui.available_width()/2.0)-(SPACE*3.0),
					_ => (ui.available_width()/3.0)-(SPACE*3.0),
				};
				ui.group(|ui| {
					ui.set_enabled(self.diff);
					let width = width / 2.0;
					if key.is_r() && !wants_input || ui.add_sized([width, height], Button::new("Reset")).on_hover_text("Reset changes").clicked() {
						let og = self.og.lock().unwrap().clone();
						self.state.gupax = og.gupax;
						self.state.p2pool = og.p2pool;
						self.state.xmrig = og.xmrig;
						self.node_vec = self.og_node_vec.clone();
						self.pool_vec = self.og_pool_vec.clone();
					}
					if key.is_s() && !wants_input || ui.add_sized([width, height], Button::new("Save")).on_hover_text("Save changes").clicked() {
						match State::save(&mut self.state, &self.state_path) {
							Ok(_) => {
								let mut og = self.og.lock().unwrap();
								og.gupax = self.state.gupax.clone();
								og.p2pool = self.state.p2pool.clone();
								og.xmrig = self.state.xmrig.clone();
							},
							Err(e) => {
								self.error_state.set(format!("State file: {}", e), ErrorFerris::Error, ErrorButtons::Okay);
							},
						};
						match Node::save(&self.node_vec, &self.node_path) {
							Ok(_) => self.og_node_vec = self.node_vec.clone(),
							Err(e) => self.error_state.set(format!("Node list: {}", e), ErrorFerris::Error, ErrorButtons::Okay),
						};
						match Pool::save(&self.pool_vec, &self.pool_path) {
							Ok(_) => self.og_pool_vec = self.pool_vec.clone(),
							Err(e) => self.error_state.set(format!("Pool list: {}", e), ErrorFerris::Error, ErrorButtons::Okay),
						};
					}
				});

				// [Simple/Advanced] + [Start/Stop/Restart]
				match self.tab {
					Tab::Gupax => {
						ui.group(|ui| {
							let width = width / 2.0;
							if ui.add_sized([width, height], SelectableLabel::new(!self.state.gupax.simple, "Advanced")).on_hover_text(GUPAX_ADVANCED).clicked() {
								self.state.gupax.simple = false;
							}
							ui.separator();
							if ui.add_sized([width, height], SelectableLabel::new(self.state.gupax.simple, "Simple")).on_hover_text(GUPAX_SIMPLE).clicked() {
								self.state.gupax.simple = true;
							}
						});
					},
					Tab::P2pool => {
						ui.group(|ui| {
							let width = width / 1.5;
							if ui.add_sized([width, height], SelectableLabel::new(!self.state.p2pool.simple, "Advanced")).on_hover_text(P2POOL_ADVANCED).clicked() {
								self.state.p2pool.simple = false;
							}
							ui.separator();
							if ui.add_sized([width, height], SelectableLabel::new(self.state.p2pool.simple, "Simple")).on_hover_text(P2POOL_SIMPLE).clicked() {
								self.state.p2pool.simple = true;
							}
						});
						ui.group(|ui| {
							let width = (ui.available_width()/3.0)-5.0;
							if p2pool_is_waiting {
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("⟲")).on_disabled_hover_text("Restart P2Pool");
									ui.add_sized([width, height], Button::new("⏹")).on_disabled_hover_text("Stop P2Pool");
									ui.add_sized([width, height], Button::new("▶")).on_disabled_hover_text("Start P2Pool");
								});
							} else if p2pool_is_alive {
								if key.is_up() && !wants_input || ui.add_sized([width, height], Button::new("⟲")).on_hover_text("Restart P2Pool").clicked() {
									Helper::restart_p2pool(&self.helper, &self.state.p2pool, &self.state.gupax.absolute_p2pool_path);
								}
								if key.is_down() && !wants_input || ui.add_sized([width, height], Button::new("⏹")).on_hover_text("Stop P2Pool").clicked() {
									Helper::stop_p2pool(&self.helper);
								}
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("▶")).on_disabled_hover_text("Start P2Pool");
								});
							} else {
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("⟲")).on_disabled_hover_text("Restart P2Pool");
									ui.add_sized([width, height], Button::new("⏹")).on_disabled_hover_text("Stop P2Pool");
								});
								// Check if address is okay before allowing to start.
								let mut text = String::new();
								let mut ui_enabled = true;
								if !Regexes::addr_ok(&self.regex, &self.state.p2pool.address) {
									ui_enabled = false;
									text = P2POOL_ADDRESS.to_string();
								} else if !Gupax::path_is_file(&self.state.gupax.p2pool_path) {
									ui_enabled = false;
									text = P2POOL_PATH_NOT_FILE.to_string();
								} else if !crate::update::check_p2pool_path(&self.state.gupax.p2pool_path) {
									ui_enabled = false;
									text = P2POOL_PATH_NOT_VALID.to_string();
								}
								ui.set_enabled(ui_enabled);
								if (ui_enabled && key.is_up() && !wants_input) || ui.add_sized([width, height], Button::new("▶")).on_hover_text("Start P2Pool").on_disabled_hover_text(text).clicked() {
									Helper::start_p2pool(&self.helper, &self.state.p2pool, &self.state.gupax.absolute_p2pool_path);
								}
							}
						});
					},
					Tab::Xmrig => {
						ui.group(|ui| {
							let width = width / 1.5;
							if ui.add_sized([width, height], SelectableLabel::new(!self.state.xmrig.simple, "Advanced")).on_hover_text(XMRIG_ADVANCED).clicked() {
								self.state.xmrig.simple = false;
							}
							ui.separator();
							if ui.add_sized([width, height], SelectableLabel::new(self.state.xmrig.simple, "Simple")).on_hover_text(XMRIG_SIMPLE).clicked() {
								self.state.xmrig.simple = true;
							}
						});
						ui.group(|ui| {
							let width = (ui.available_width()/3.0)-5.0;
							if xmrig_is_waiting {
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("⟲")).on_disabled_hover_text("Restart XMRig");
									ui.add_sized([width, height], Button::new("⏹")).on_disabled_hover_text("Stop XMRig");
									ui.add_sized([width, height], Button::new("▶")).on_disabled_hover_text("Start XMRig");
								});
							} else if xmrig_is_alive {
								if key.is_up() && !wants_input || ui.add_sized([width, height], Button::new("⟲")).on_hover_text("Restart XMRig").clicked() {
									if cfg!(windows) {
										Helper::restart_xmrig(&self.helper, &self.state.xmrig, &self.state.gupax.absolute_xmrig_path, Arc::clone(&self.sudo));
									} else {
										self.sudo.lock().unwrap().signal = ProcessSignal::Restart;
										self.error_state.ask_sudo(&self.sudo);
									}
								}
								if key.is_down() && !wants_input || ui.add_sized([width, height], Button::new("⏹")).on_hover_text("Stop XMRig").clicked() {
									if cfg!(target_os = "macos") {
										self.sudo.lock().unwrap().signal = ProcessSignal::Stop;
										self.error_state.ask_sudo(&self.sudo);
									} else {
										Helper::stop_xmrig(&self.helper);
									}
								}
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("▶")).on_disabled_hover_text("Start XMRig");
								});
							} else {
								ui.add_enabled_ui(false, |ui| {
									ui.add_sized([width, height], Button::new("⟲")).on_disabled_hover_text("Restart XMRig");
									ui.add_sized([width, height], Button::new("⏹")).on_disabled_hover_text("Stop XMRig");
								});
								let mut text = String::new();
								let mut ui_enabled = true;
								if !Gupax::path_is_file(&self.state.gupax.xmrig_path) {
									ui_enabled = false;
									text = XMRIG_PATH_NOT_FILE.to_string();
								} else if !crate::update::check_xmrig_path(&self.state.gupax.xmrig_path) {
									ui_enabled = false;
									text = XMRIG_PATH_NOT_VALID.to_string();
								}
								ui.set_enabled(ui_enabled);
								#[cfg(target_os = "windows")]
								if (ui_enabled && key.is_up() && !wants_input) || ui.add_sized([width, height], Button::new("▶")).on_hover_text("Start XMRig").on_disabled_hover_text(text).clicked() {
									Helper::start_xmrig(&self.helper, &self.state.xmrig, &self.state.gupax.absolute_xmrig_path, Arc::clone(&self.sudo));
								}
								#[cfg(target_family = "unix")]
								if (ui_enabled && key.is_up() && !wants_input) || ui.add_sized([width, height], Button::new("▶")).on_hover_text("Start XMRig").on_disabled_hover_text(text).clicked() {
									self.sudo.lock().unwrap().signal = ProcessSignal::Start;
									self.error_state.ask_sudo(&self.sudo);
								}
							}
						});
					},
					_ => (),
				}
			});
			});
		});

		// Middle panel, contents of the [Tab]
		debug!("App | Rendering CENTRAL_PANEL (tab contents)");
		CentralPanel::default().show(ctx, |ui| {
			// This sets the Ui dimensions after Top/Bottom are filled
			self.width = ui.available_width();
			self.height = ui.available_height();
			ui.style_mut().override_text_style = Some(TextStyle::Body);
			match self.tab {
				Tab::About => {
					debug!("App | Entering [About] Tab");
					let width = self.width;
					let height = self.height/30.0;
					let max_height = self.height;
					ui.add_space(10.0);
					ui.vertical_centered(|ui| {
						ui.set_max_height(max_height);
						// Display [Gupax] banner
						self.img.banner.show_max_size(ui, Vec2::new(width, height*3.0));
						ui.add_sized([width, height], Label::new("Gupax is a cross-platform GUI for mining"));
						ui.add_sized([width, height], Hyperlink::from_label_and_url("[Monero]", "https://www.github.com/monero-project/monero"));
						ui.add_sized([width, height], Label::new("on"));
						ui.add_sized([width, height], Hyperlink::from_label_and_url("[P2Pool]", "https://www.github.com/SChernykh/p2pool"));
						ui.add_sized([width, height], Label::new("using"));
						ui.add_sized([width, height], Hyperlink::from_label_and_url("[XMRig]", "https://www.github.com/xmrig/xmrig"));

						ui.add_space(SPACE*3.0);
						ui.style_mut().override_text_style = Some(Monospace);
						ui.add_sized([width, height], Label::new(KEYBOARD_SHORTCUTS));
						ui.style_mut().override_text_style = Some(Body);
						ui.add_space(SPACE*3.0);

						ui.add_sized([width, height], Hyperlink::from_label_and_url("Powered by egui", "https://github.com/emilk/egui"));
						ui.add_sized([width, height], Hyperlink::from_label_and_url("Made by hinto-janaiyo".to_string(), "https://gupax.io"));
						ui.add_sized([width, height], Label::new("egui is licensed under MIT & Apache-2.0"));
						ui.add_sized([width, height], Label::new("Gupax, P2Pool, and XMRig are licensed under GPLv3"));
						if cfg!(debug_assertions) { ui.label(format!("Gupax is running in debug mode - {}", self.now.elapsed().as_secs_f64())); }
					});
				}
				Tab::Status => {
					debug!("App | Entering [Status] Tab");
					Status::show(&self.pub_sys, &self.p2pool_api, &self.xmrig_api, &self.p2pool_img, &self.xmrig_img, p2pool_is_alive, xmrig_is_alive, self.width, self.height, ctx, ui);
				}
				Tab::Gupax => {
					debug!("App | Entering [Gupax] Tab");
					Gupax::show(&mut self.state.gupax, &self.og, &self.state_path, &self.update, &self.file_window, &mut self.error_state, &self.restart, self.width, self.height, frame, ctx, ui);
				}
				Tab::P2pool => {
					debug!("App | Entering [P2Pool] Tab");
					P2pool::show(&mut self.state.p2pool, &mut self.node_vec, &self.og, &self.ping, &self.regex, &self.p2pool, &self.p2pool_api, &mut self.p2pool_console, self.width, self.height, ctx, ui);
				}
				Tab::Xmrig => {
					debug!("App | Entering [XMRig] Tab");
					Xmrig::show(&mut self.state.xmrig, &mut self.pool_vec, &self.regex, &self.xmrig, &self.xmrig_api, &mut self.xmrig_console, self.width, self.height, ctx, ui);
				}
			}
		});
	}
}
