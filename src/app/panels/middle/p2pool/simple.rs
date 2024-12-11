use std::sync::Arc;
use std::sync::Mutex;

use crate::app::panels::middle::ProgressBar;
use crate::components::node::Ping;
use crate::components::node::RemoteNode;
use crate::components::node::format_ip_location;
use crate::components::node::format_ms;
use crate::disk::state::P2pool;
use crate::miscs::height_txt_before_button;
use egui::Button;
use egui::Checkbox;
use egui::ScrollArea;
use egui::TextStyle;
use egui::TextWrapMode;
use egui::vec2;

use crate::constants::*;
use egui::{Color32, ComboBox, RichText, Ui};
use log::*;
impl P2pool {
    pub(super) fn simple(&mut self, ui: &mut Ui, ping: &Arc<Mutex<Ping>>) {
        ui.vertical_centered(|ui|{
            ui.add_space(SPACE);
            ui.checkbox(&mut self.local_node, "Use a local node").on_hover_text("If checked (recommended), p2pool will automatically use the local node.\nCheck the Node tab to start a local node.\nIf unchecked, p2pool will attempt to use a remote node.");
        });
        ui.add_space(SPACE * 2.0);
        // if checked, use only local node
        // if unchecked, show remote nodes.
        // disable remote if local is checked.
        let visible = !self.local_node;
        debug!("P2Pool Tab | Running [auto-select] check");
        if self.auto_select && visible {
            let mut ping = ping.lock().unwrap();
            // If we haven't auto_selected yet, auto-select and turn it off
            if ping.pinged && !ping.auto_selected {
                self.node = ping.fastest.to_string();
                ping.auto_selected = true;
            }
            drop(ping);
        }
        ui.add_enabled_ui(visible, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    debug!("P2Pool Tab | Rendering [Ping List]");
                    // [Ping List]
                    let mut ms = 0;
                    let mut color = Color32::LIGHT_GRAY;
                    if ping.lock().unwrap().pinged {
                        for data in ping.lock().unwrap().nodes.iter() {
                            if data.ip == self.node {
                                ms = data.ms;
                                color = data.color;
                                break;
                            }
                        }
                    }
                    debug!("P2Pool Tab | Rendering [ComboBox] of Remote Nodes");
                    let ip_location = format_ip_location(&self.node, false);
                    let text = RichText::new(format!(" ⏺ {}ms | {}", ms, ip_location)).color(color);
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ComboBox::from_id_salt("remote_nodes")
                        .selected_text(text)
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            for data in ping.lock().unwrap().nodes.iter() {
                                let ms = format_ms(data.ms);
                                let ip_location = format_ip_location(data.ip, true);
                                let text = RichText::new(format!(" ⏺ {} | {}", ms, ip_location))
                                    .color(data.color);
                                ui.selectable_value(&mut self.node, data.ip.to_string(), text);
                            }
                        });
                });
                ui.add_space(SPACE);
                debug!("P2Pool Tab | Rendering [Select fastest ... Ping] buttons");
                ScrollArea::horizontal()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .id_salt("horizontal")
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Button);
                        ui.horizontal(|ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            // ui.columns_const(|[col1, col2, col3, col4, col5]| {
                            let width = ((ui.available_width() / 5.0)
                                - (ui.spacing().item_spacing.x * (4.0 / 5.0)))
                                .max(20.0);
                            let height = height_txt_before_button(ui, &TextStyle::Button) * 2.0;
                            // [Select random node]
                            ui.style_mut().override_text_valign = Some(egui::Align::Center);
                            if ui
                                .add_sized([width, height], Button::new("Select random node"))
                                .on_hover_text(P2POOL_SELECT_RANDOM)
                                .clicked()
                            {
                                self.node = RemoteNode::get_random(&self.node);
                            }
                            // [Select fastest node]
                            if ui
                                .add_sized([width, height], Button::new("Select fastest node"))
                                .on_hover_text(P2POOL_SELECT_FASTEST)
                                .clicked()
                                && ping.lock().unwrap().pinged
                            {
                                self.node = ping.lock().unwrap().fastest.to_string();
                            }
                            // [Ping Button]
                            ui.add_enabled_ui(!ping.lock().unwrap().pinging, |ui| {
                                if ui
                                    .add_sized([width, height], Button::new("Ping remote nodes"))
                                    .on_hover_text(P2POOL_PING)
                                    .clicked()
                                {
                                    Ping::spawn_thread(ping);
                                }
                            });
                            // [Last <-]
                            if ui
                                .add_sized([width, height], Button::new("⬅ Last"))
                                .on_hover_text(P2POOL_SELECT_LAST)
                                .clicked()
                            {
                                let ping = ping.lock().unwrap();
                                match ping.pinged {
                                    true => {
                                        self.node =
                                            RemoteNode::get_last_from_ping(&self.node, &ping.nodes)
                                    }
                                    false => self.node = RemoteNode::get_last(&self.node),
                                }
                                drop(ping);
                            }
                            // [Next ->]
                            if ui
                                .add_sized([width, height], Button::new("Next ➡"))
                                .on_hover_text(P2POOL_SELECT_NEXT)
                                .clicked()
                            {
                                let ping = ping.lock().unwrap();
                                match ping.pinged {
                                    true => {
                                        self.node =
                                            RemoteNode::get_next_from_ping(&self.node, &ping.nodes)
                                    }
                                    false => self.node = RemoteNode::get_next(&self.node),
                                }
                                drop(ping);
                            }
                        });

                        ui.vertical_centered(|ui| {
                            // let height = height / 2.0;
                            let pinging = ping.lock().unwrap().pinging;
                            ui.add_enabled_ui(pinging, |ui| {
                                let prog = ping.lock().unwrap().prog.round();
                                let msg = RichText::new(format!(
                                    "{} ... {}%",
                                    ping.lock().unwrap().msg,
                                    prog
                                ));
                                // let height = height / 1.25;
                                // let size = vec2(size.x, height);
                                ui.add_space(SPACE);
                                ui.label(msg);
                                ui.add_space(SPACE);
                                if pinging {
                                    ui.spinner();
                                } else {
                                    ui.label("...");
                                }
                                ui.add(ProgressBar::new(prog.round() / 100.0));
                                ui.add_space(SPACE);
                            });
                        });

                        debug!("P2Pool Tab | Rendering [Auto-*] buttons");
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let width =
                                    (((ui.available_width() - ui.spacing().item_spacing.x) / 3.0)
                                        - SPACE * 1.5)
                                        .max(ui.text_style_height(&TextStyle::Button) * 7.0);
                                let size = vec2(
                                    width,
                                    height_txt_before_button(ui, &TextStyle::Button) * 2.0,
                                );
                                // [Auto-node]
                                ui.add_sized(
                                    size,
                                    Checkbox::new(&mut self.auto_select, "Auto-select"),
                                )
                                // ui.checkbox(&mut self.auto_select, "Auto-select")
                                .on_hover_text(P2POOL_AUTO_SELECT);
                                ui.separator();
                                // [Auto-node]
                                ui.add_sized(size, Checkbox::new(&mut self.auto_ping, "Auto-ping"))
                                    // ui.checkbox(&mut self.auto_ping, "Auto-ping")
                                    .on_hover_text(P2POOL_AUTO_NODE);
                                ui.separator();
                                // [Backup host]
                                ui.add_sized(
                                    size,
                                    Checkbox::new(&mut self.backup_host, "Backup host"),
                                )
                                // ui.checkbox(&mut self.backup_host, "Backup host")
                                .on_hover_text(P2POOL_BACKUP_HOST_SIMPLE);
                            })
                        });
                    });
            });
            debug!("P2Pool Tab | Rendering warning text");
            ui.add_space(SPACE);
            ui.vertical_centered(|ui| {
                ui.hyperlink_to(
                "WARNING: It is recommended to run/use your own Monero Node (hover for details)",
                "https://github.com/Cyrix126/gupaxx#running-a-local-monero-node",
            )
            .on_hover_text(P2POOL_COMMUNITY_NODE_WARNING);
            });
        });
    }
}
