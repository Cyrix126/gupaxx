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

use crate::constants::*;
use crate::disk::pool::Pool;
use crate::disk::state::Xmrig;
use crate::helper::Process;
use crate::helper::xrig::xmrig::PubXmrigApi;
use crate::regex::{REGEXES, num_lines};
use crate::utils::regex::Regexes;
use egui::{
    Button, Checkbox, ComboBox, Label, RichText, SelectableLabel, Slider, TextEdit, TextStyle,
    Vec2, vec2,
};
use log::*;

use std::sync::{Arc, Mutex};

impl Xmrig {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        pool_vec: &mut Vec<(String, Pool)>,
        process: &Arc<Mutex<Process>>,
        api: &Arc<Mutex<PubXmrigApi>>,
        buffer: &mut String,
        size: Vec2,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        let text_edit = size.y / 25.0;
        //---------------------------------------------------------------------------------------------------- [Simple] Console
        debug!("XMRig Tab | Rendering [Console]");
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.group(|ui| {
            let text = &api.lock().unwrap().output;
            let nb_lines = num_lines(text);
            let (height, width) = if self.simple {
                (size.y / 1.5, size.x - SPACE)
            } else {
                (size.y / 2.8, size.x - SPACE)
            };
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                ui.style_mut().override_text_style = Some(TextStyle::Small);
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_width(width)
                    .max_height(height)
                    .auto_shrink([false; 2])
                    // .show_viewport(ui, |ui, _| {
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Small),
                        nb_lines,
                        |ui, row_range| {
                            for i in row_range {
                                if let Some(line) = text.lines().nth(i) {
                                    ui.label(line);
                                }
                            }
                        },
                    );
            });
            //---------------------------------------------------------------------------------------------------- [Advanced] Console
            if !self.simple {
                ui.separator();
                let response = ui
                    .add_sized(
                        [width, text_edit],
                        TextEdit::hint_text(
                            TextEdit::singleline(buffer),
                            r#"Commands: [h]ashrate, [p]ause, [r]esume, re[s]ults, [c]onnection"#,
                        ),
                    )
                    .on_hover_text(XMRIG_INPUT);
                // If the user pressed enter, dump buffer contents into the process STDIN
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    response.request_focus(); // Get focus back
                    let buffer = std::mem::take(buffer); // Take buffer
                    let mut process = process.lock().unwrap(); // Lock
                    if process.is_alive() {
                        process.input.push(buffer);
                    } // Push only if alive
                }
            }
        });

        //---------------------------------------------------------------------------------------------------- Arguments
        if !self.simple {
            debug!("XMRig Tab | Rendering [Arguments]");
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let width = (size.x / 10.0) - SPACE;
                    ui.add_sized([width, text_edit], Label::new("Command arguments:"));
                    ui.add_sized(
                        [ui.available_width(), text_edit],
                        TextEdit::hint_text(
                            TextEdit::singleline(&mut self.arguments),
                            r#"--url <...> --user <...> --config <...>"#,
                        ),
                    )
                    .on_hover_text(XMRIG_ARGUMENTS);
                    self.arguments.truncate(1024);
                })
            });
            ui.add_enabled_ui(self.arguments.is_empty(), |ui|{

            //---------------------------------------------------------------------------------------------------- Address
            debug!("XMRig Tab | Rendering [Address]");
            ui.group(|ui| {
                let width = size.x - SPACE;
                ui.spacing_mut().text_edit_width = (width) - (SPACE * 3.0);
                let text;
                let color;
                let len = format!("{:02}", self.address.len());
                if self.address.is_empty() {
                    text = format!("Monero Address [{}/95] ➖", len);
                    color = LIGHT_GRAY;
                } else if Regexes::addr_ok(&self.address) {
                    text = format!("Monero Address [{}/95] ✔", len);
                    color = GREEN;
                } else {
                    text = format!("Monero Address [{}/95] ❌", len);
                    color = RED;
                }
                ui.add_sized(
                    [width, text_edit],
                    Label::new(RichText::new(text).color(color)),
                );
                ui.add_sized(
                    [width, text_edit],
                    TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4..."),
                )
                .on_hover_text(XMRIG_ADDRESS);
                self.address.truncate(95);
            });
            });
        }

        //---------------------------------------------------------------------------------------------------- Threads
        if self.simple {
            ui.add_space(SPACE);
        }
        debug!("XMRig Tab | Rendering [Threads]");
        ui.vertical(|ui| {
            let width = size.x / 10.0;
            let text_width = width * 2.4;
            ui.spacing_mut().slider_width = width * 6.5;
            ui.spacing_mut().icon_width = width / 25.0;
            ui.horizontal(|ui| {
                ui.add_sized(
                    [text_width, text_edit],
                    Label::new(format!("Threads [1-{}]:", self.max_threads)),
                );
                ui.add_sized(
                    [width, text_edit],
                    Slider::new(&mut self.current_threads, 1..=self.max_threads),
                )
                .on_hover_text(XMRIG_THREADS);
            });
            #[cfg(not(target_os = "linux"))] // Pause on active isn't supported on Linux
            ui.horizontal(|ui| {
                ui.add_sized(
                    [text_width, text_edit],
                    Label::new("Pause on active [0-255]:".to_string()),
                );
                ui.add_sized([width, text_edit], Slider::new(&mut self.pause, 0..=255))
                    .on_hover_text(format!("{} [{}] seconds.", XMRIG_PAUSE, self.pause));
            });
        });

        //---------------------------------------------------------------------------------------------------- Simple
        if !self.simple {
            debug!("XMRig Tab | Rendering [Pool List] elements");
            let width = ui.available_width() - 10.0;
            let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
                                             // [Pool IP/Port]
            ui.horizontal(|ui| {
		ui.group(|ui| {
			let width = width/10.0;
			ui.vertical(|ui| {
			ui.spacing_mut().text_edit_width = width*3.32;
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.name.len());
				if self.name.is_empty() {
					text = format!("Name [ {}/30 ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.name.is_match(&self.name) {
					text = format!("Name [ {}/30 ]✔", len);
					color = GREEN;
				} else {
					text = format!("Name [ {}/30 ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.name).on_hover_text(XMRIG_NAME);
				self.name.truncate(30);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.ip.len());
				if self.ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if self.ip == "localhost" || REGEXES.ipv4.is_match(&self.ip) || REGEXES.domain.is_match(&self.ip) {
					text = format!("  IP [{}/255]✔", len);
					color = GREEN;
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.ip).on_hover_text(XMRIG_IP);
				self.ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.port.len();
				if self.port.is_empty() {
					text = format!("Port [  {}/5  ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.port) {
					text = format!("Port [  {}/5  ]✔", len);
					color = GREEN;
				} else {
					text = format!("Port [  {}/5  ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.port).on_hover_text(XMRIG_PORT);
				self.port.truncate(5);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.rig.len());
				if self.rig.is_empty() {
					text = format!(" Rig [ {}/30 ]➖", len);
					color = LIGHT_GRAY;
				} else if REGEXES.name.is_match(&self.rig) {
					text = format!(" Rig [ {}/30 ]✔", len);
					color = GREEN;
				} else {
					text = format!(" Rig [ {}/30 ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.rig).on_hover_text(XMRIG_RIG);
				self.rig.truncate(30);
			});
		});

		ui.vertical(|ui| {
			let width = ui.available_width();
			ui.add_space(1.0);
			// [Manual node selection]
			ui.spacing_mut().slider_width = width - 8.0;
			ui.spacing_mut().icon_width = width / 25.0;
			// [Node List]
			debug!("XMRig Tab | Rendering [Node List] ComboBox");
			let text = RichText::new(format!("{}. {}", self.selected_index+1, self.selected_name));
			ComboBox::from_id_salt("manual_pool").selected_text(text).width(width).show_ui(ui, |ui| {
				for (n, (name, pool)) in pool_vec.iter().enumerate() {
					let text = format!("{}. {}\n     IP: {}\n   Port: {}\n    Rig: {}", n+1, name, pool.ip, pool.port, pool.rig);
					if ui.add(SelectableLabel::new(self.selected_name == *name, text)).clicked() {
						self.selected_index = n;
						let pool = pool.clone();
						self.selected_name.clone_from(name);
						self.selected_rig.clone_from(&pool.rig);
						self.selected_ip.clone_from(&pool.ip);
						self.selected_port.clone_from(&pool.port);
						self.name.clone_from(name);
						self.rig = pool.rig;
						self.ip = pool.ip;
						self.port = pool.port;
					}
				}
			});
			// [Add/Save]
			let pool_vec_len = pool_vec.len();
			let mut exists = false;
			let mut save_diff = true;
			let mut existing_index = 0;
			for (name, pool) in pool_vec.iter() {
				if *name == self.name {
					exists = true;
					if self.rig == pool.rig && self.ip == pool.ip && self.port == pool.port {
						save_diff = false;
					}
					break
				}
				existing_index += 1;
			}
			ui.horizontal(|ui| {
				let text = if exists { LIST_SAVE } else { LIST_ADD };
				let text = format!("{}\n    Currently selected pool: {}. {}\n    Current amount of pools: {}/1000", text, self.selected_index+1, self.selected_name, pool_vec_len);
				// If the pool already exists, show [Save] and mutate the already existing pool
				if exists {
					ui.add_enabled_ui(!incorrect_input && save_diff, |ui|{
					if ui.add_sized([width, text_edit], Button::new("Save")).on_hover_text(text).clicked() {
						let pool = Pool {
							rig: self.rig.clone(),
							ip: self.ip.clone(),
							port: self.port.clone(),
						};
						pool_vec[existing_index].1 = pool;
						self.selected_name.clone_from(&self.name);
						self.selected_rig.clone_from(&self.rig);
						self.selected_ip.clone_from(&self.ip);
						self.selected_port.clone_from(&self.port);
						info!("Node | S | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", existing_index+1, self.name, self.ip, self.port, self.rig);
					}
					});
				// Else, add to the list
				} else {
					ui.add_enabled_ui(!incorrect_input && pool_vec_len < 1000, |ui|{
					if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
						let pool = Pool {
							rig: self.rig.clone(),
							ip: self.ip.clone(),
							port: self.port.clone(),
						};
						pool_vec.push((self.name.clone(), pool));
						self.selected_index = pool_vec_len;
						self.selected_name.clone_from(&self.name);
						self.selected_rig.clone_from(&self.rig);
						self.selected_ip.clone_from(&self.ip);
						self.selected_port.clone_from(&self.port);
						info!("Node | A | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", pool_vec_len, self.name, self.ip, self.port, self.rig);
					}
					});
				}
			});
			// [Delete]
			ui.horizontal(|ui| {
				ui.add_enabled_ui(pool_vec_len > 1, |ui|{
					let text = format!("{}\n    Currently selected pool: {}. {}\n    Current amount of pools: {}/1000", LIST_DELETE, self.selected_index+1, self.selected_name, pool_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Delete")).on_hover_text(text).clicked() {
					let new_name;
					let new_pool;
					match self.selected_index {
						0 => {
							new_name = pool_vec[1].0.clone();
							new_pool = pool_vec[1].1.clone();
							pool_vec.remove(0);
						}
						_ => {
							pool_vec.remove(self.selected_index);
							self.selected_index -= 1;
							new_name = pool_vec[self.selected_index].0.clone();
							new_pool = pool_vec[self.selected_index].1.clone();
						}
					};
					self.selected_name.clone_from(&new_name);
					self.selected_rig.clone_from(&new_pool.rig);
					self.selected_ip.clone_from(&new_pool.ip);
					self.selected_port.clone_from(&new_pool.port);
					self.name = new_name;
					self.rig = new_pool.rig;
					self.ip = new_pool.ip;
					self.port = new_pool.port;
					info!("Node | D | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig\"{}\"]", self.selected_index, self.selected_name, self.selected_ip, self.selected_port, self.selected_rig);
}
				});

			});
			ui.horizontal(|ui| {
				ui.add_enabled_ui(!self.name.is_empty() || !self.ip.is_empty() || !self.port.is_empty(), |ui|{
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(LIST_CLEAR).clicked() {
					self.name.clear();
					self.rig.clear();
					self.ip.clear();
					self.port.clear();
				}
				});
			});
		});
		});
		});
            ui.add_space(5.0);

            debug!("XMRig Tab | Rendering [API] TextEdits");
            // [HTTP API IP/Port]
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let width = width / 10.0;
                        ui.spacing_mut().text_edit_width = width * 2.39;
                        // HTTP API
                        ui.horizontal(|ui| {
                            let text;
                            let color;
                            let len = format!("{:03}", self.api_ip.len());
                            if self.api_ip.is_empty() {
                                text = format!("HTTP API IP   [{}/255]➖", len);
                                color = LIGHT_GRAY;
                                incorrect_input = true;
                            } else if self.api_ip == "localhost"
                                || REGEXES.ipv4.is_match(&self.api_ip)
                                || REGEXES.domain.is_match(&self.api_ip)
                            {
                                text = format!("HTTP API IP   [{}/255]✔", len);
                                color = GREEN;
                            } else {
                                text = format!("HTTP API IP   [{}/255]❌", len);
                                color = RED;
                                incorrect_input = true;
                            }
                            ui.add_sized(
                                [width, text_edit],
                                Label::new(RichText::new(text).color(color)),
                            );
                            ui.text_edit_singleline(&mut self.api_ip)
                                .on_hover_text(XMRIG_API_IP);
                            self.api_ip.truncate(255);
                        });
                        ui.horizontal(|ui| {
                            let text;
                            let color;
                            let len = self.api_port.len();
                            if self.api_port.is_empty() {
                                text = format!("HTTP API Port [  {}/5  ]➖", len);
                                color = LIGHT_GRAY;
                                incorrect_input = true;
                            } else if REGEXES.port.is_match(&self.api_port) {
                                text = format!("HTTP API Port [  {}/5  ]✔", len);
                                color = GREEN;
                            } else {
                                text = format!("HTTP API Port [  {}/5  ]❌", len);
                                color = RED;
                                incorrect_input = true;
                            }
                            ui.add_sized(
                                [width, text_edit],
                                Label::new(RichText::new(text).color(color)),
                            );
                            ui.text_edit_singleline(&mut self.api_port)
                                .on_hover_text(XMRIG_API_PORT);
                            self.api_port.truncate(5);
                        });
                    });

                    ui.separator();

                    debug!("XMRig Tab | Rendering [TLS/Keepalive] buttons");
                    ui.vertical(|ui| {
                        // TLS/Keepalive
                        ui.horizontal(|ui| {
                            let width = (ui.available_width() / 2.0) - 11.0;
                            let height = text_edit * 2.0;
                            let size = vec2(width, height);
                            //				let mut style = (*ctx.style()).clone();
                            //				style.spacing.icon_width_inner = width / 8.0;
                            //				style.spacing.icon_width = width / 6.0;
                            //				style.spacing.icon_spacing = 20.0;
                            //				ctx.set_style(style);
                            ui.add_sized(size, Checkbox::new(&mut self.tls, "TLS Connection"))
                                .on_hover_text(XMRIG_TLS);
                            ui.separator();
                            ui.add_sized(size, Checkbox::new(&mut self.keepalive, "Keepalive"))
                                .on_hover_text(XMRIG_KEEPALIVE);
                        });
                    });
                });
            });
        }
    });
    }
}
