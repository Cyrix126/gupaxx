use crate::disk::node::Node;
use crate::{disk::state::P2pool, utils::regex::REGEXES};
use egui::Checkbox;
use egui::Slider;
use egui::{Button, Vec2};

use crate::constants::*;
use egui::{Color32, ComboBox, Label, RichText, SelectableLabel, TextStyle::*, Ui};
use log::*;

impl P2pool {
    pub(super) fn advanced(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        text_edit: f32,
        node_vec: &mut Vec<(String, Node)>,
    ) {
        let height = size.y / 16.0;
        let space_h = size.y / 128.0;
        debug!("P2Pool Tab | Rendering [Node List] elements");
        let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
                                         // [Monero node IP/RPC/ZMQ]
        ui.horizontal(|ui| {
		    ui.group(|ui| {
			let width = size.x/10.0;
			ui.vertical(|ui| {
			ui.spacing_mut().text_edit_width = width*3.32;
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.name.len());
				if self.name.is_empty() {
					text = format!("Name [ {}/30 ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.name.is_match(&self.name) {
					text = format!("Name [ {}/30 ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("Name [ {}/30 ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.name).on_hover_text(P2POOL_NAME);
				self.name.truncate(30);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.ip.len());
				if self.ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if self.ip == "localhost" || REGEXES.ipv4.is_match(&self.ip) || REGEXES.domain.is_match(&self.ip) {
					text = format!("  IP [{}/255]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.ip).on_hover_text(P2POOL_NODE_IP);
				self.ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.rpc.len();
				if self.rpc.is_empty() {
					text = format!(" RPC [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.rpc) {
					text = format!(" RPC [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" RPC [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.rpc).on_hover_text(P2POOL_RPC_PORT);
				self.rpc.truncate(5);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.zmq.len();
				if self.zmq.is_empty() {
					text = format!(" ZMQ [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.zmq) {
					text = format!(" ZMQ [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" ZMQ [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.zmq).on_hover_text(P2POOL_ZMQ_PORT);
				self.zmq.truncate(5);
			});
		});

		ui.vertical(|ui| {
			let width = ui.available_width();
			ui.add_space(1.0);
			// [Manual node selection]
			ui.spacing_mut().slider_width = width - 8.0;
			ui.spacing_mut().icon_width = width / 25.0;
			// [Ping List]
			debug!("P2Pool Tab | Rendering [Node List]");
			let text = RichText::new(format!("{}. {}", self.selected_index+1, self.selected_name));
			ComboBox::from_id_source("manual_nodes").selected_text(text).width(width).show_ui(ui, |ui| {
				for (n, (name, node)) in node_vec.iter().enumerate() {
					let text = RichText::new(format!("{}. {}\n     IP: {}\n    RPC: {}\n    ZMQ: {}", n+1, name, node.ip, node.rpc, node.zmq));
					if ui.add(SelectableLabel::new(self.selected_name == *name, text)).clicked() {
						self.selected_index = n;
						let node = node.clone();
						self.selected_name.clone_from(name);
						self.selected_ip.clone_from(&node.ip);
						self.selected_rpc.clone_from(&node.rpc);
						self.selected_zmq.clone_from(&node.zmq);
						self.name.clone_from(name);
						self.ip = node.ip;
						self.rpc = node.rpc;
						self.zmq = node.zmq;
					}
				}
			});
			// [Add/Save]
			let node_vec_len = node_vec.len();
			let mut exists = false;
			let mut save_diff = true;
			let mut existing_index = 0;
			for (name, node) in node_vec.iter() {
				if *name == self.name {
					exists = true;
					if self.ip == node.ip && self.rpc == node.rpc && self.zmq == node.zmq {
						save_diff = false;
					}
					break
				}
				existing_index += 1;
			}
			ui.horizontal(|ui| {
				let text = if exists { LIST_SAVE } else { LIST_ADD };
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", text, self.selected_index+1, self.selected_name, node_vec_len);
				// If the node already exists, show [Save] and mutate the already existing node
				if exists {
					ui.set_enabled(!incorrect_input && save_diff);
					if ui.add_sized([width, text_edit], Button::new("Save")).on_hover_text(text).clicked() {
						let node = Node {
							ip: self.ip.clone(),
							rpc: self.rpc.clone(),
							zmq: self.zmq.clone(),
						};
						node_vec[existing_index].1 = node;
						self.selected_index = existing_index;
						self.selected_ip.clone_from(&self.ip);
						self.selected_rpc.clone_from(&self.rpc);
						self.selected_zmq.clone_from(&self.zmq);
						info!("Node | S | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", existing_index+1, self.name, self.ip, self.rpc, self.zmq);
					}
				// Else, add to the list
				} else {
					ui.set_enabled(!incorrect_input && node_vec_len < 1000);
					if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
						let node = Node {
							ip: self.ip.clone(),
							rpc: self.rpc.clone(),
							zmq: self.zmq.clone(),
						};
						node_vec.push((self.name.clone(), node));
						self.selected_index = node_vec_len;
						self.selected_name.clone_from(&self.name);
						self.selected_ip.clone_from(&self.ip);
						self.selected_rpc.clone_from(&self.rpc);
						self.selected_zmq.clone_from(&self.zmq);
						info!("Node | A | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", node_vec_len, self.name, self.ip, self.rpc, self.zmq);
					}
				}
			});
			// [Delete]
			ui.horizontal(|ui| {
				ui.set_enabled(node_vec_len > 1);
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", LIST_DELETE, self.selected_index+1, self.selected_name, node_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Delete")).on_hover_text(text).clicked() {
					let new_name;
					let new_node;
					match self.selected_index {
						0 => {
							new_name = node_vec[1].0.clone();
							new_node = node_vec[1].1.clone();
							node_vec.remove(0);
						}
						_ => {
							node_vec.remove(self.selected_index);
							self.selected_index -= 1;
							new_name = node_vec[self.selected_index].0.clone();
							new_node = node_vec[self.selected_index].1.clone();
						}
					};
					self.selected_name.clone_from(&new_name);
					self.selected_ip.clone_from(&new_node.ip);
					self.selected_rpc.clone_from(&new_node.rpc);
					self.selected_zmq.clone_from(&new_node.zmq);
					self.name = new_name;
					self.ip = new_node.ip;
					self.rpc = new_node.rpc;
					self.zmq = new_node.zmq;
					info!("Node | D | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", self.selected_index, self.selected_name, self.selected_ip, self.selected_rpc, self.selected_zmq);
				}
			});
			ui.horizontal(|ui| {
				ui.set_enabled(!self.name.is_empty() || !self.ip.is_empty() || !self.rpc.is_empty() || !self.zmq.is_empty());
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(LIST_CLEAR).clicked() {
					self.name.clear();
					self.ip.clear();
					self.rpc.clear();
					self.zmq.clear();
				}
			});
		});
		});
		});
        // ui.add_space(space_h);

        debug!("P2Pool Tab | Rendering [Main/Mini/Peers/Log] elements");
        // [Main/Mini]
        ui.horizontal(|ui| {
            let height = height / 4.0;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let width = (size.x / 4.0) - SPACE;
                    let height = height + space_h;
                    if ui
                        .add_sized(
                            [width, height],
                            SelectableLabel::new(!self.mini, "P2Pool Main"),
                        )
                        .on_hover_text(P2POOL_MAIN)
                        .clicked()
                    {
                        self.mini = false;
                    }
                    if ui
                        .add_sized(
                            [width, height],
                            SelectableLabel::new(self.mini, "P2Pool Mini"),
                        )
                        .on_hover_text(P2POOL_MINI)
                        .clicked()
                    {
                        self.mini = true;
                    }
                })
            });
            // [Out/In Peers] + [Log Level]
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let text = (ui.available_width() / 10.0) - SPACE;
                    let width = (text * 8.0) - SPACE;
                    let height = height / 3.0;
                    ui.style_mut().spacing.slider_width = width / 1.1;
                    ui.style_mut().spacing.interact_size.y = height;
                    ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                    ui.horizontal(|ui| {
                        ui.add_sized([text, height], Label::new("Out peers [10-450]:"));
                        ui.add_sized([width, height], Slider::new(&mut self.out_peers, 10..=450))
                            .on_hover_text(P2POOL_OUT);
                        ui.add_space(ui.available_width() - 4.0);
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized([text, height], Label::new(" In peers [10-450]:"));
                        ui.add_sized([width, height], Slider::new(&mut self.in_peers, 10..=450))
                            .on_hover_text(P2POOL_IN);
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized([text, height], Label::new("   Log level [0-6]:"));
                        ui.add_sized([width, height], Slider::new(&mut self.log_level, 0..=6))
                            .on_hover_text(P2POOL_LOG);
                    });
                })
            });
        });

        debug!("P2Pool Tab | Rendering Backup host button");
        ui.group(|ui| {
            let width = size.x - SPACE;
            let height = ui.available_height();
            ui.style_mut().spacing.icon_width = height;
            ui.style_mut().spacing.icon_width_inner = height * 0.9;
            // [Backup host]
            ui.add_sized(
                [width, height],
                Checkbox::new(&mut self.backup_host, "Backup host"),
            )
            .on_hover_text(P2POOL_BACKUP_HOST_ADVANCED);
        });
    }
}
