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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

//---------------------------------------------------------------------------------------------------- Imports
// egui/eframe
use egui::TextStyle::*;
use egui::color::Color32;
use egui::FontFamily::Proportional;
use egui::{FontId,Label,RichText,Stroke,Vec2,Pos2};
use egui_extras::RetainedImage;
use eframe::{egui,NativeOptions};

// Logging
use log::*;
use env_logger::{Builder,WriteStyle};

// std
use std::io::Write;
use std::process::exit;
use std::sync::{Arc,Mutex};
use std::{thread,env};
use std::time::Instant;
use std::path::PathBuf;

// Modules
mod constants;
mod node;
mod state;
mod about;
mod status;
mod gupax;
mod p2pool;
mod xmrig;
use {constants::*,node::*,state::*,about::*,status::*,gupax::*,p2pool::*,xmrig::*};

//---------------------------------------------------------------------------------------------------- Struct + Impl
// The state of the outer main [App].
// See the [State] struct in [state.rs] for the
// actual inner state of the tab settings.
pub struct App {
	// Misc state
	tab: Tab, // What tab are we on?
	quit: bool, // Was the quit button clicked?
	quit_confirm: bool, // Was the quit confirmed?
	ping: bool, // Was the ping button clicked?
	pinging: Arc<Mutex<bool>>, // Is a ping in progress?
	node: Arc<Mutex<NodeStruct>>, // Data on community nodes
	// State:
	// og    = Old state to compare against
	// state = Working state (current settings)
	// Instead of comparing [og == state] every frame,
	// the [diff] bool will be the signal for [Reset/Save].
	og: State,
	state: State,
	diff: bool,
	// Process/update state:
	// Doesn't make sense to save this on disk
	// so it's represented as a bool here.
	p2pool: bool, // Is p2pool online?
	xmrig: bool, // Is xmrig online?
	updating: bool, // Is an update in progress?
	// State from [--flags]
	startup: bool,
	reset: bool,
	// Static stuff
	now: Instant, // Internal timer
	resolution: Vec2, // Frame resolution
	os: &'static str, // OS
	version: String, // Gupax version
	name_version: String, // [Gupax vX.X.X]
	banner: RetainedImage, // Gupax banner image
}

impl App {
	fn cc(cc: &eframe::CreationContext<'_>, app: Self) -> Self {
		let resolution = cc.integration_info.window_info.size;
		init_text_styles(&cc.egui_ctx, resolution[0] as f32);
		Self {
			resolution,
			..app
		}
	}

	fn default() -> Self {
		let app = Self {
			tab: Tab::default(),
			quit: false,
			quit_confirm: false,
			ping: false,
			pinging: Arc::new(Mutex::new(false)),
			node: Arc::new(Mutex::new(NodeStruct::default())),
			og: State::default(),
			state: State::default(),
			diff: false,
			p2pool: false,
			xmrig: false,
			updating: false,
			startup: true,
			reset: false,
			now: Instant::now(),
			resolution: Vec2::new(1280.0, 720.0),
			os: OS,
			version: "v0.0.1".to_string(),
			name_version: "Gupax v0.0.1".to_string(),
			banner: RetainedImage::from_image_bytes("banner.png", BYTES_BANNER).expect("oops"),
		};
		parse_args(app)
	}
}

//---------------------------------------------------------------------------------------------------- Enum + Impl
// The tabs inside [App].
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tab {
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

//---------------------------------------------------------------------------------------------------- Init functions
fn init_text_styles(ctx: &egui::Context, width: f32) {
	let scale = width / 26.666;
	let mut style = (*ctx.style()).clone();
	style.text_styles = [
//		(Small, FontId::new(10.0, Proportional)),
//		(Body, FontId::new(25.0, Proportional)),
//		(Button, FontId::new(25.0, Proportional)),
//		(Monospace, FontId::new(25.0, Proportional)),
//		(Heading, FontId::new(30.0, Proportional)),
//		(Name("Tab".into()), FontId::new(50.0, Proportional)),
//		(Name("Bottom".into()), FontId::new(25.0, Proportional)),
		(Small, FontId::new(scale/3.0, Proportional)),
		(Body, FontId::new(scale/2.0, Proportional)),
		(Button, FontId::new(scale/2.0, Proportional)),
		(Monospace, FontId::new(scale/2.0, Proportional)),
		(Heading, FontId::new(scale/1.5, Proportional)),
		(Name("Tab".into()), FontId::new(scale*1.2, Proportional)),
		(Name("Bottom".into()), FontId::new(scale/2.0, Proportional)),
	].into();
//	style.spacing.slider_width = scale;
//	style.spacing.text_edit_width = scale;
//	style.spacing.button_padding = Vec2::new(scale/2.0, scale/2.0);
	ctx.set_style(style);
	ctx.set_pixels_per_point(1.0);
	ctx.request_repaint();
}

fn init_logger() {
	#[cfg(debug_assertions)]
	let filter = LevelFilter::Info;
	#[cfg(not(debug_assertions))]
	let filter = LevelFilter::Warn;
	use env_logger::fmt::Color;
	Builder::new().format(|buf, record| {
		let level;
		let mut style = buf.style();
		match record.level() {
			Level::Error => { style.set_color(Color::Red); level = "ERROR" },
			Level::Warn => { style.set_color(Color::Yellow); level = "WARN" },
			Level::Info => { style.set_color(Color::White); level = "INFO" },
			Level::Debug => { style.set_color(Color::Blue); level = "DEBUG" },
			Level::Trace => { style.set_color(Color::Magenta); level = "TRACE" },
		};
		writeln!(
			buf,
			"[{}] [{}] [{}:{}] {}",
			style.set_bold(true).value(level),
			buf.style().set_dimmed(true).value(chrono::Local::now().format("%F %T%.3f")),
			buf.style().set_dimmed(true).value(record.file().unwrap_or("???")),
			buf.style().set_dimmed(true).value(record.line().unwrap_or(0)),
			record.args(),
		)
	}).filter_level(filter).write_style(WriteStyle::Always).parse_default_env().format_timestamp_millis().init();
	info!("init_logger() ... OK");
}

fn init_options() -> NativeOptions {
	let mut options = eframe::NativeOptions::default();
	options.min_window_size = Option::from(Vec2::new(1280.0, 720.0));
	options.max_window_size = Option::from(Vec2::new(3180.0, 2160.0));
	options.initial_window_size = Option::from(Vec2::new(1280.0, 720.0));
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

//---------------------------------------------------------------------------------------------------- Misc functions
fn into_absolute_path(path: String) -> Result<PathBuf, std::io::Error> {
	let path = PathBuf::from(path);
	if path.is_relative() {
		let mut dir = std::env::current_exe()?;
		dir.pop();
		dir.push(path);
		Ok(dir)
	} else {
		Ok(path)
	}
}

fn parse_args(mut app: App) -> App {
	info!("Parsing CLI arguments...");
	let mut args: Vec<String> = env::args().collect();
	if args.len() == 1 { info!("No args ... OK"); return app } else { args.remove(0); info!("Args ... {:?}", args); }
	// [help/version], exit early
	for arg in &args {
		match arg.as_str() {
			"-h"|"--help"    => { println!("{}", ARG_HELP); exit(0); },
			"-v"|"--version" => {
				println!("Gupax  | {}\nP2Pool | {}\nXMRig  | {}\n\n{}", GUPAX_VERSION, P2POOL_VERSION, XMRIG_VERSION, ARG_COPYRIGHT);
				exit(0);
			},
			_ => (),
		}
	}
	// Everything else
	for arg in args {
		match arg.as_str() {
			"-n"|"--no-startup" => { info!("Disabling startup..."); app.startup = false; }
			"-r"|"--reset" => { info!("Resetting state..."); app.reset = true; }
			_ => { eprintln!("[Gupax error] Invalid option: [{}]\nFor help, use: [--help]", arg); exit(1); },
		}
	}
	app
}

//---------------------------------------------------------------------------------------------------- [App] frame for [Panic] situations
struct Panic { error_msg: String, }
impl Panic {
	fn options() -> NativeOptions {
		let mut options = eframe::NativeOptions::default();
		let frame = Option::from(Vec2::new(1280.0, 720.0));
		options.min_window_size = frame;
		options.max_window_size = frame;
		options.initial_window_size = frame;
		options.follow_system_theme = false;
		options.default_theme = eframe::Theme::Dark;
		let icon = image::load_from_memory(BYTES_ICON).expect("Failed to read icon bytes").to_rgba8();
		let (icon_width, icon_height) = icon.dimensions();
		options.icon_data = Some(eframe::IconData {
			rgba: icon.into_raw(),
			width: icon_width,
			height: icon_height,
		});
		info!("Panic::options() ... OK");
		options
	}
	fn new(cc: &eframe::CreationContext<'_>, error_msg: String) -> Self {
		let resolution = cc.integration_info.window_info.size;
		init_text_styles(&cc.egui_ctx, resolution[0] as f32);
		Self { error_msg }
	}
}

impl eframe::App for Panic {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			let width = ui.available_width();
			let height = ui.available_height();
			init_text_styles(ctx, width);
			ui.add_sized([width, height/8.0], Label::new("Gupax has encountered a fatal error:"));
			ui.add_sized([width, height/8.0], Label::new(&self.error_msg));
			ui.add_sized([width, height/3.0], Label::new("Please report to: https://github.com/hinto-janaiyo/gupax/issues"));
			ui.add_sized([width, height/3.0], egui::Button::new("Quit")).clicked() && exit(1)
		});
	}
}

//---------------------------------------------------------------------------------------------------- Main [App] frame
fn main() {
	init_logger();
	let app = App::default();
	let options = init_options();
	let toml = match State::get() {
		Ok(toml) => toml,
		Err(err) => {
			error!("{}", err);
			let error_msg = err.to_string();
			let options = Panic::options();
			eframe::run_native("Gupax", options, Box::new(|cc| Box::new(Panic::new(cc, error_msg))),);
			exit(1);
		},
	};
	eframe::run_native("Gupax", options, Box::new(|cc| Box::new(App::cc(cc, app))),);
}

impl eframe::App for App {
	fn on_close_event(&mut self) -> bool {
		self.quit = true;
		self.quit_confirm
	}

	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//		init_text_styles(ctx, 1280.0);

		// Close confirmation.
		if self.quit {
			egui::CentralPanel::default().show(ctx, |ui| {
				init_text_styles(ctx, ui.available_width());
				let width = ui.available_width();
				let width = width - 10.0;
				let height = ui.available_height();
				// Detect processes or update
				if self.p2pool || self.xmrig {
					ui.add_sized([width, height/6.0], Label::new("Are you sure you want to quit?"));
					if self.p2pool { ui.add_sized([width, height/6.0], Label::new("P2Pool is online...!")); }
					if self.xmrig { ui.add_sized([width, height/6.0], Label::new("XMRig is online...!")); }
				// Else, just quit
				} else {
					if self.state.gupax.save_before_quit {
						info!("Saving before quit...");
						match self.state.save() {
							Err(err) => { error!("{}", err); exit(1); },
							_ => (),
						};
					}
					info!("No processes or update in progress ... goodbye!");
					exit(0);
				}
				egui::TopBottomPanel::bottom("quit").show(ctx, |ui| {
					ui.group(|ui| {
						if ui.add_sized([width, height/8.0], egui::Button::new("Yes")).clicked() {
							if self.state.gupax.save_before_quit {
								info!("Saving before quit...");
								match self.state.save() {
									Err(err) => { error!("{}", err); exit(1); },
									_ => (),
								};
							}
							info!("Quit confirmation = yes ... goodbye!");
							self.quit_confirm = true;
							exit(0);
						} else if ui.add_sized([width, height/8.0], egui::Button::new("No")).clicked() {
							self.quit = false;
						}
					});
				});
			});
			return
		}

		// If ping was pressed, start thread
		if self.ping {
			self.ping = false;
			self.pinging = Arc::new(Mutex::new(true));
			let node_clone = Arc::clone(&self.node);
			let pinging_clone = Arc::clone(&self.pinging);
			thread::spawn(move|| {
				let result = NodeStruct::ping();
				*node_clone.lock().unwrap() = result.nodes;
				*pinging_clone.lock().unwrap() = false;
			});
		}

		// If ping-ING, display stats
//		if *self.pinging.lock().unwrap() {
//			egui::CentralPanel::default().show(ctx, |ui| {
//				let width = ui.available_width();
//				let width = width - 10.0;
//				let height = ui.available_height();
//				init_text_styles(ctx, width);
//				ui.add_sized([width, height/2.0], Label::new(format!("In progress: {}", *self.pinging.lock().unwrap())));
//				ui.group(|ui| {
//					if ui.add_sized([width, height/10.0], egui::Button::new("Yes")).clicked() {
//						info!("Quit confirmation = yes ... goodbye!");
//						exit(0);
//					} else if ui.add_sized([width, height/10.0], egui::Button::new("No")).clicked() {
//						info!("Quit confirmation = no ... returning!");
//						self.show_confirmation_dialog = false;
//					}
//				});
//			});
//			return
//		}

		// Top: Tabs
		egui::CentralPanel::default().show(ctx, |ui| {
			init_text_styles(ctx, ui.available_width());
			let width = (ui.available_width() - 90.0) / 5.0;
			let height = ui.available_height() / 10.0;
		    ui.add_space(4.0);
			ui.horizontal(|ui| {
				ui.style_mut().override_text_style = Some(Name("Tab".into()));
				ui.style_mut().visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(100, 100, 100);
				ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(255, 120, 120);
				ui.style_mut().visuals.selection.stroke = Stroke {
					width: 5.0,
					color: Color32::from_rgb(255, 255, 255),
				};
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::About, "About")).clicked() { self.tab = Tab::About; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Status, "Status")).clicked() { self.tab = Tab::Status; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Gupax, "Gupax")).clicked() { self.tab = Tab::Gupax; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::P2pool, "P2Pool")).clicked() { self.tab = Tab::P2pool; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Xmrig, "XMRig")).clicked() { self.tab = Tab::Xmrig; }
			});
			ui.add_space(3.0);
			ui.separator();
//		});

		let height = height / 2.0;
		// Bottom: app info + state/process buttons
		egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
			ui.style_mut().override_text_style = Some(Name("Bottom".into()));
			ui.horizontal(|ui| {
				ui.group(|ui| {
					let width = width / 2.0;
					ui.add_sized([width, height], Label::new(&self.name_version));
					ui.separator();
					ui.add_sized([width, height], Label::new(self.os));
					ui.separator();
					ui.add_sized([width/1.5, height], Label::new("P2Pool"));
					if self.p2pool == true {
						ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(100, 230, 100))));
					} else {
						ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(230, 50, 50))));
					}
					ui.separator();
					ui.add_sized([width/1.5, height], Label::new("XMRig"));
					if self.xmrig == true {
							ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(100, 230, 100))));
					} else {
							ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(230, 50, 50))));
					}
				});

				ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
					ui.group(|ui| {
						if self.state == self.og {
							ui.set_enabled(false)
						}
						let width = width / 2.0;
						if ui.add_sized([width, height], egui::Button::new("Save")).on_hover_text("Save changes").clicked() { self.og = self.state.clone(); }
						if ui.add_sized([width, height], egui::Button::new("Reset")).on_hover_text("Reset changes").clicked() { self.state = self.og.clone(); }
					});

					let width = (ui.available_width() / 3.0) - 6.2;
					match self.tab {
						Tab::P2pool => {
							ui.group(|ui| {
								if self.p2pool == true {
									if ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart P2Pool").clicked() { self.p2pool = false; }
									if ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop P2Pool").clicked() { self.p2pool = false; }
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start P2Pool");
									});
								} else {
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart P2Pool");
										ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop P2Pool");
									});
									if ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start P2Pool").clicked() { self.p2pool = true; }
								}
							});
						}
						Tab::Xmrig => {
							ui.group(|ui| {
								if self.xmrig == true {
									if ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart XMRig").clicked() { self.xmrig = false; }
									if ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop XMRig").clicked() { self.xmrig = false; }
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start XMRig");
									});
								} else {
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart XMRig");
										ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop XMRig");
									});
									if ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start XMRig").clicked() { self.xmrig = true; }
								}
							});
						}
						_ => (),
					}
				});
			});
		});

		ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
        match self.tab {
            Tab::About => {
				About::show(self, ctx, ui);
            }
            Tab::Status => {
				Status::show(self, ctx, ui);
            }
            Tab::Gupax => {
//				Gupax::show(self.state.gupax, ctx, ui);
				exit(0);
            }
            Tab::P2pool => {
				P2pool::show(&mut self.state.p2pool, ctx, ui);
            }
            Tab::Xmrig => {
				Xmrig::show(&mut self.state.xmrig, ctx, ui);
            }
        }
		});
	}
}
