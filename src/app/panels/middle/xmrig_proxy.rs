use egui::{vec2, Button, Checkbox, ComboBox, Label, RichText, SelectableLabel, TextEdit, Vec2};
use std::sync::{Arc, Mutex};

use egui::TextStyle::{self, Name};
use log::{debug, info};

use crate::disk::pool::Pool;
use crate::disk::state::XmrigProxy;
use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::helper::Process;
use crate::regex::{num_lines, REGEXES};
use crate::utils::constants::DARK_GRAY;
use crate::utils::macros::lock;
use crate::{
    GREEN, LIGHT_GRAY, LIST_ADD, LIST_CLEAR, LIST_DELETE, LIST_SAVE, RED, SPACE, XMRIG_API_IP,
    XMRIG_API_PORT, XMRIG_IP, XMRIG_KEEPALIVE, XMRIG_NAME, XMRIG_PORT, XMRIG_PROXY_ARGUMENTS,
    XMRIG_PROXY_INPUT, XMRIG_PROXY_REDIRECT, XMRIG_PROXY_URL, XMRIG_RIG, XMRIG_TLS,
};

impl XmrigProxy {
    #[inline(always)] // called once
    pub fn show(
        &mut self,
        process: &Arc<Mutex<Process>>,
        pool_vec: &mut Vec<(String, Pool)>,
        api: &Arc<Mutex<PubXmrigProxyApi>>,
        buffer: &mut String,
        size: Vec2,
        ui: &mut egui::Ui,
    ) {
        let width = size.x;
        let height = size.y;
        let space_h = height / 48.0;
        let text_edit = size.y / 25.0;
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(space_h);
            ui.style_mut().override_text_style = Some(TextStyle::Heading);
            ui.hyperlink_to("XMRig-Proxy", XMRIG_PROXY_URL);
            ui.style_mut().override_text_style = Some(TextStyle::Body);
            ui.add(Label::new("High performant proxy for your miners"));
            ui.add_space(space_h);
        });
        // console output for log
        debug!("XvB Tab | Rendering [Console]");
        ui.group(|ui| {
            let text = &lock!(api).output;
            let nb_lines = num_lines(text);
            let height = size.y / 2.8;
            let width = size.x - (space_h / 2.0);
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_width(width)
                    .max_height(height)
                    .auto_shrink([false; 2])
                    // .show_viewport(ui, |ui, _| {
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Name("MonospaceSmall".into())),
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
        });
        //---------------------------------------------------------------------------------------------------- [Advanced] Console
        if !self.simple {
            ui.separator();
            let response = ui
                .add_sized(
                    [width, text_edit],
                    TextEdit::hint_text(
                        TextEdit::singleline(buffer),
                        r#"Commands: [h]ashrate, [c]onnections, [v]erbose, [w]orkers"#,
                    ),
                )
                .on_hover_text(XMRIG_PROXY_INPUT);
            // If the user pressed enter, dump buffer contents into the process STDIN
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                response.request_focus(); // Get focus back
                let buffer = std::mem::take(buffer); // Take buffer
                let mut process = lock!(process); // Lock
                if process.is_alive() {
                    process.input.push(buffer);
                } // Push only if alive
            }

            //---------------------------------------------------------------------------------------------------- Arguments
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
                    .on_hover_text(XMRIG_PROXY_ARGUMENTS);
                    self.arguments.truncate(1024);
                })
            });
            if !self.arguments.is_empty() {
            	ui.disable();
            }
            ui.add_space(space_h);
            ui.style_mut().spacing.icon_width_inner = width / 45.0;
            ui.style_mut().spacing.icon_width = width / 35.0;
            ui.style_mut().spacing.icon_spacing = space_h;
            ui.checkbox(
                &mut self.redirect_local_xmrig,
                "Auto Redirect local Xmrig to Xmrig-Proxy",
            )
            .on_hover_text(XMRIG_PROXY_REDIRECT);

            // idea
            // need to warn the user if local firewall is blocking port
            // need to warn the user if NAT is blocking port
            // need to show local ip address
            // need to show public ip

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
				let len = format!("{:03}", self.p2pool_ip.len());
				if self.p2pool_ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if self.p2pool_ip == "localhost" || REGEXES.ipv4.is_match(&self.p2pool_ip) || REGEXES.domain.is_match(&self.p2pool_ip) {
					text = format!("  IP [{}/255]✔", len);
					color = GREEN;
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.p2pool_ip).on_hover_text(XMRIG_IP);
				self.p2pool_ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.p2pool_port.len();
				if self.p2pool_port.is_empty() {
					text = format!("Port [  {}/5  ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.p2pool_port) {
					text = format!("Port [  {}/5  ]✔", len);
					color = GREEN;
				} else {
					text = format!("Port [  {}/5  ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.p2pool_port).on_hover_text(XMRIG_PORT);
				self.p2pool_port.truncate(5);
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
			ComboBox::from_id_source("manual_pool").selected_text(text).width(width).show_ui(ui, |ui| {
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
						self.p2pool_ip = pool.ip;
						self.p2pool_port = pool.port;
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
					if self.rig == pool.rig && self.p2pool_ip == pool.ip && self.p2pool_port == pool.port {
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
							ip: self.p2pool_ip.clone(),
							port: self.p2pool_port.clone(),
						};
						pool_vec[existing_index].1 = pool;
						self.selected_name.clone_from(&self.name);
						self.selected_rig.clone_from(&self.rig);
						self.selected_ip.clone_from(&self.p2pool_ip);
						self.selected_port.clone_from(&self.p2pool_port);
						info!("Node | S | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", existing_index+1, self.name, self.p2pool_ip, self.p2pool_port, self.rig);
					}

					});
				// Else, add to the list
				} else {
					ui.add_enabled_ui(!incorrect_input && pool_vec_len < 1000, |ui|{
					if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
						let pool = Pool {
							rig: self.rig.clone(),
							ip: self.p2pool_ip.clone(),
							port: self.p2pool_port.clone(),
						};
						pool_vec.push((self.name.clone(), pool));
						self.selected_index = pool_vec_len;
						self.selected_name.clone_from(&self.name);
						self.selected_rig.clone_from(&self.rig);
						self.selected_ip.clone_from(&self.p2pool_ip);
						self.selected_port.clone_from(&self.p2pool_port);
						info!("Node | A | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", pool_vec_len, self.name, self.p2pool_ip, self.p2pool_port, self.rig);
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
					self.p2pool_ip = new_pool.ip;
					self.p2pool_port = new_pool.port;
					info!("Node | D | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig\"{}\"]", self.selected_index, self.selected_name, self.selected_ip, self.selected_port, self.selected_rig);
				}
				});
			});
			ui.horizontal(|ui| {
				ui.add_enabled_ui(!self.name.is_empty() || !self.p2pool_ip.is_empty() || !self.p2pool_port.is_empty(), |ui|{
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(LIST_CLEAR).clicked() {
					self.name.clear();
					self.rig.clear();
					self.p2pool_ip.clear();
					self.p2pool_port.clear();
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
