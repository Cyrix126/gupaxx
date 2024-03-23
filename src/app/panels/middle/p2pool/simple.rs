use std::sync::Arc;
use std::sync::Mutex;

use crate::app::panels::middle::Hyperlink;
use crate::app::panels::middle::ProgressBar;
use crate::app::panels::middle::Spinner;
use crate::components::node::format_ip_location;
use crate::components::node::format_ms;
use crate::components::node::Ping;
use crate::components::node::RemoteNode;
use crate::disk::state::P2pool;
use crate::utils::macros::lock;
use egui::vec2;
use egui::Button;
use egui::Checkbox;
use egui::Vec2;

use crate::constants::*;
use egui::{Color32, ComboBox, Label, RichText, Ui};
use log::*;
impl P2pool {
    pub(super) fn simple(&mut self, ui: &mut Ui, size: Vec2, ping: &Arc<Mutex<Ping>>) {
        // [Node]
        let height = size.y / 13.0;
        let space_h = size.y / 96.0;
        ui.spacing_mut().slider_width = size.x - 16.0;
        ui.spacing_mut().icon_width = size.x / 25.0;

        // [Auto-select] if we haven't already.
        // Using [Arc<Mutex<Ping>>] as an intermediary here
        // saves me the hassle of wrapping [state: State] completely
        // and [.lock().unwrap()]ing it everywhere.
        // Two atomic bools = enough to represent this data
        debug!("P2Pool Tab | Running [auto-select] check");
        if self.auto_select {
            let mut ping = lock!(ping);
            // If we haven't auto_selected yet, auto-select and turn it off
            if ping.pinged && !ping.auto_selected {
                self.node = ping.fastest.to_string();
                ping.auto_selected = true;
            }
            drop(ping);
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                debug!("P2Pool Tab | Rendering [Ping List]");
                // [Ping List]
                let mut ms = 0;
                let mut color = Color32::LIGHT_GRAY;
                if lock!(ping).pinged {
                    for data in lock!(ping).nodes.iter() {
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
                ComboBox::from_id_source("remote_nodes")
                    .selected_text(text)
                    .width(size.x)
                    .show_ui(ui, |ui| {
                        for data in lock!(ping).nodes.iter() {
                            let ms = format_ms(data.ms);
                            let ip_location = format_ip_location(data.ip, true);
                            let text = RichText::new(format!(" ⏺ {} | {}", ms, ip_location))
                                .color(data.color);
                            ui.selectable_value(&mut self.node, data.ip.to_string(), text);
                        }
                    });
            });

            ui.add_space(space_h);

            debug!("P2Pool Tab | Rendering [Select fastest ... Ping] buttons");
            ui.horizontal(|ui| {
                let width = (size.x / 5.0) - 6.0;
                let size = vec2(width, height);
                // [Select random node]
                if ui
                    .add_sized(size, Button::new("Select random node"))
                    .on_hover_text(P2POOL_SELECT_RANDOM)
                    .clicked()
                {
                    self.node = RemoteNode::get_random(&self.node);
                }
                // [Select fastest node]
                if ui
                    .add_sized(size, Button::new("Select fastest node"))
                    .on_hover_text(P2POOL_SELECT_FASTEST)
                    .clicked()
                    && lock!(ping).pinged
                {
                    self.node = lock!(ping).fastest.to_string();
                }
                // [Ping Button]
                ui.add_enabled_ui(!lock!(ping).pinging, |ui| {
                    if ui
                        .add_sized(size, Button::new("Ping remote nodes"))
                        .on_hover_text(P2POOL_PING)
                        .clicked()
                    {
                        Ping::spawn_thread(ping);
                    }
                });
                // [Last <-]
                if ui
                    .add_sized(size, Button::new("⬅ Last"))
                    .on_hover_text(P2POOL_SELECT_LAST)
                    .clicked()
                {
                    let ping = lock!(ping);
                    match ping.pinged {
                        true => self.node = RemoteNode::get_last_from_ping(&self.node, &ping.nodes),
                        false => self.node = RemoteNode::get_last(&self.node),
                    }
                    drop(ping);
                }
                // [Next ->]
                if ui
                    .add_sized(size, Button::new("Next ➡"))
                    .on_hover_text(P2POOL_SELECT_NEXT)
                    .clicked()
                {
                    let ping = lock!(ping);
                    match ping.pinged {
                        true => self.node = RemoteNode::get_next_from_ping(&self.node, &ping.nodes),
                        false => self.node = RemoteNode::get_next(&self.node),
                    }
                    drop(ping);
                }
            });

            ui.vertical(|ui| {
                let height = height / 2.0;
                let pinging = lock!(ping).pinging;
                ui.set_enabled(pinging);
                let prog = lock!(ping).prog.round();
                let msg = RichText::new(format!("{} ... {}%", lock!(ping).msg, prog));
                let height = height / 1.25;
                let size = vec2(size.x, height);
                ui.add_space(space_h);
                ui.add_sized(size, Label::new(msg));
                ui.add_space(space_h);
                if pinging {
                    ui.add_sized(size, Spinner::new().size(height));
                } else {
                    ui.add_sized(size, Label::new("..."));
                }
                ui.add_sized(size, ProgressBar::new(prog.round() / 100.0));
                ui.add_space(space_h);
            });
        });

        debug!("P2Pool Tab | Rendering [Auto-*] buttons");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let width = (size.x / 3.0) - (SPACE * 1.75);
                let size = vec2(width, height);
                // [Auto-node]
                ui.add_sized(size, Checkbox::new(&mut self.auto_select, "Auto-select"))
                    .on_hover_text(P2POOL_AUTO_SELECT);
                ui.separator();
                // [Auto-node]
                ui.add_sized(size, Checkbox::new(&mut self.auto_ping, "Auto-ping"))
                    .on_hover_text(P2POOL_AUTO_NODE);
                ui.separator();
                // [Backup host]
                ui.add_sized(size, Checkbox::new(&mut self.backup_host, "Backup host"))
                    .on_hover_text(P2POOL_BACKUP_HOST_SIMPLE);
            })
        });

        debug!("P2Pool Tab | Rendering warning text");
        ui.add_sized(
            [size.x, height / 2.0],
            Hyperlink::from_label_and_url(
                "WARNING: It is recommended to run/use your own Monero Node (hover for details)",
                "https://github.com/Cyrix126/gupaxx#running-a-local-monero-node",
            ),
        )
        .on_hover_text(P2POOL_COMMUNITY_NODE_WARNING);
    }
}
