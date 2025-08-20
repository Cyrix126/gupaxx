// Gupaxx - Fork of Gupax
//
// Copyright (c) 2024-2025 Cyrix126
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

use std::sync::Arc;
use std::sync::Mutex;

use crate::app::panels::middle::ProgressBar;
use crate::app::panels::middle::common::size_calculator::height_dropdown;
use crate::components::node::Ping;
use crate::components::node::RemoteNode;
use crate::components::node::format_ms;
use crate::constants::*;
use crate::disk::state::P2pool;
use crate::helper::crawler::Crawler;
use crate::miscs::height_txt_before_button;
use egui::Align;
use egui::Button;
use egui::Checkbox;
use egui::ScrollArea;
use egui::TextStyle;
use egui::TextWrapMode;
use egui::vec2;
use egui::{ComboBox, RichText, Ui};
use log::*;

use super::p2pool::PubP2poolApi;

impl P2pool {
    pub(super) fn simple(
        &mut self,
        ui: &mut Ui,
        ping: &Arc<Mutex<Ping>>,
        api: &mut PubP2poolApi,
        crawler: &Arc<Mutex<Crawler>>,
    ) {
        ui.vertical_centered(|ui|{
            ui.add_space(SPACE);
            ui.checkbox(&mut self.local_node, "Start with a local node").on_hover_text("If checked (recommended), p2pool will start trying to use the local node.\nCheck the Node tab to start a local node.\nIf unchecked, p2pool will attempt to use a remote node.");
        });

        ui.add_space(SPACE * 2.0);
        // if checked, use only local node
        // if unchecked, show remote nodes.
        // disable remote if local is checked.
        let visible_remote_nodes = !self.local_node;
        debug!("P2Pool Tab | Running [auto-select] check");
        if self.auto_select && visible_remote_nodes {
            // If we haven't auto_selected yet, auto-select and turn it off
            let pinged = ping.lock().unwrap().pinged;
            let auto_select_ping = ping.lock().unwrap().auto_selected;
            let fastest_node = ping.lock().unwrap().nodes.first().cloned();
            if pinged
                && auto_select_ping
                && let Some(fastest_node) = fastest_node
            {
                self.selected_remote_node = Some(fastest_node);
                ping.lock().unwrap().auto_selected = true;
            }
        }

        ui.add_enabled_ui(visible_remote_nodes, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE);
                let crawling = crawler.lock().unwrap().crawling;
                let button_crawl_text = if crawling {
                    "Stop finding P2Pool compatible Nodes"
                } else {
                    "Start finding P2Pool compatible Nodes"
                };
                if ui
                    .button(button_crawl_text)
                    .on_hover_text("It will reset the current found nodes")
                    .clicked()
                {
                    if !crawling {
                        self.selected_remote_node = None;
                        Crawler::start(crawler);
                    } else {
                        crawler.lock().unwrap().msg = "Stopped manually".to_string();
                        Crawler::stop(crawler);
                    }
                }
                crawl_progress(crawler, ui);
            });
            ui.vertical(|ui| {
                let crawler_lock = crawler.lock().unwrap();
                let mut ping_lock = ping.lock().unwrap();
                // [Ping List]
                //
                //
                // Refresh the ping list with crawled nodes
                let ping_nodes = &mut ping_lock.nodes;
                let crawl_nodes = &crawler_lock.nodes;

                if *ping_nodes != *crawl_nodes && !crawl_nodes.is_empty() {
                    *ping_nodes = crawl_nodes.clone();
                }

                // refresh the selected node with the fastest from the pinged nodes if it was empty
                if self.selected_remote_node.is_none() {
                    self.selected_remote_node = ping_nodes.first().cloned();
                }
                drop(ping_lock);
                let crawling = crawler_lock.crawling;
                drop(crawler_lock);
                self.list_nodes(ping, crawling, ui);
                ui.add_space(SPACE);
                self.list_nodes_buttons(ping, ui);
                ui.vertical_centered(|ui| {
                    ping_progress(ping, ui);
                });
                self.auto_buttons(api, ui);

                warning_should_run_local_node(ui);
            });
        });
    }

    fn list_nodes(&mut self, ping: &Arc<Mutex<Ping>>, crawling: bool, ui: &mut Ui) {
        // don't show anything if there is no nodes nor selected_saved node
        if let Some(selected_saved_node) = self.selected_remote_node.as_mut() {
            // Take the first (fastest) node to replace the selected saved node if it is not present in the found nodes or if the crawler is running.
            // If the crawler is running, we consider the user doesn't care about the last selected node, so we update it to be the fastest.
            // If the crawler is not running, the selected node can be changed for the fastest if it is not in the list.
            let ping_nodes = &mut ping.lock().unwrap().nodes;

            // refresh the selected node with the fastest found
            if crawling && let Some(node) = ping_nodes.first() {
                *selected_saved_node = node.clone();
            }

            // if the selected does not exist anymore, take the fastest
            if !ping_nodes.iter().any(|n| n == selected_saved_node)
                && let Some(node) = ping_nodes.first()
            {
                *selected_saved_node = node.clone();
            }

            // if there is no nodes found, add it to list so it can get pinged
            if ping_nodes.is_empty() {
                dbg!("before adding {ping_nodes}");
                ping_nodes.push(selected_saved_node.clone());
                dbg!("after adding {ping_nodes}");
            }

            // the selected node saved in the state file will not include the latency
            // The real data must be taken from the ping nodes
            // If the selected node is not present in ping, takes the data from state
            let selected_node = if let Some(node) = ping_nodes.find_selected(selected_saved_node) {
                node
            } else {
                &selected_saved_node.clone()
            };

            // If some node are found, make the selectable list visible
            ui.horizontal(|ui| {
                debug!("P2Pool Tab | Rendering [Ping List]");

                debug!("P2Pool Tab | Rendering [ComboBox] of Remote Nodes");
                let ping_msg = if selected_node.ms == 0 {
                    "Latency not measured".to_string()
                } else {
                    format_ms(selected_node.ms).to_string()
                };
                let country = selected_node.country();
                let text = RichText::new(format!(" ⏺ {ping_msg} | {country}"))
                    .color(selected_node.ping_color());
                ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.style_mut().override_text_valign = Some(Align::Center);
                ui.set_height(0.0);
                ComboBox::from_id_salt("remote_nodes")
                    .selected_text(text)
                    .width(ui.available_width())
                    .height(height_dropdown(ping_nodes.len(), ui) * 10.0)
                    .show_ui(ui, |ui| {
                        for data in ping_nodes.iter() {
                            let country = data.country();
                            let ping_msg = if data.ms == 0 {
                                "Latency not measured".to_string()
                            } else {
                                format_ms(data.ms).to_string()
                            };

                            let text = RichText::new(format!(" ⏺ {ping_msg} | {country}"))
                                .color(data.ping_color());
                            ui.selectable_value(selected_saved_node, data.clone(), text);
                        }
                    });
            });
        }
    }
    fn list_nodes_buttons(&mut self, ping: &Arc<Mutex<Ping>>, ui: &mut Ui) {
        if let Some(selected_node) = self.selected_remote_node.as_mut() {
            debug!("P2Pool Tab | Rendering [Select fastest ... Ping] buttons");
            ScrollArea::horizontal()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .id_salt("horizontal")
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Button);
                    ui.horizontal(|ui| {
                        // style of buttons
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let width = ((ui.available_width() / 5.0)
                            - (ui.spacing().item_spacing.x * (4.0 / 5.0)))
                            .max(20.0);
                        let height = height_txt_before_button(ui, &TextStyle::Button) * 2.0;
                        ui.style_mut().override_text_valign = Some(egui::Align::Center);

                        // [Select random node]
                        select_random_node_button(ping, selected_node, width, height, ui);
                        // [Select fastest node]
                        select_fastest_node_button(ping, selected_node, width, height, ui);
                        // [Ping Button]
                        ping_button(ping, width, height, ui);
                        // [Last <-]
                        last_node_button(width, height, selected_node, ping, ui);
                        // [Next ->]
                        next_node_button(width, height, selected_node, ping, ui);
                    });
                });
        }
    }
    fn auto_buttons(&mut self, api: &mut PubP2poolApi, ui: &mut Ui) {
        debug!("P2Pool Tab | Rendering [Auto-*] buttons");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let width = (((ui.available_width() - ui.spacing().item_spacing.x) / 4.0)
                    - SPACE * 1.5)
                    .max(ui.text_style_height(&TextStyle::Button) * 7.0);
                let size = vec2(
                    width,
                    height_txt_before_button(ui, &TextStyle::Button) * 2.0,
                );
                ui.add_sized(size, Checkbox::new(&mut self.auto_select, "Auto-select"))
                    .on_hover_text(P2POOL_AUTO_SELECT);
                ui.separator();
                ui.add_sized(size, Checkbox::new(&mut self.auto_ping, "Auto-ping"))
                    .on_hover_text(P2POOL_AUTO_NODE);
                ui.separator();
                ui.add_sized(size, Checkbox::new(&mut self.backup_host, "Backup host"))
                    .on_hover_text(P2POOL_BACKUP_HOST_SIMPLE);
                ui.separator();
                // set preferred local node immediately if we are on simple mode.
                if ui
                    .add_sized(
                        size,
                        Checkbox::new(&mut self.prefer_local_node, "Auto-Switch to Local Node"),
                    )
                    .on_hover_text(P2POOL_AUTOSWITCH_LOCAL_NODE)
                    .clicked()
                {
                    api.prefer_local_node = self.prefer_local_node;
                }
            })
        });
    }
}
fn warning_should_run_local_node(ui: &mut Ui) {
    debug!("P2Pool Tab | Rendering warning text");
    ui.add_space(SPACE);
    ui.vertical_centered(|ui| {
        ui.hyperlink_to(
            "WARNING: It is recommended to run/use your own Monero Node (hover for details)",
            "https://github.com/Cyrix126/gupaxx#running-a-local-monero-node",
        )
        .on_hover_text(P2POOL_COMMUNITY_NODE_WARNING);
    });
}
fn crawl_progress(crawler: &Arc<Mutex<Crawler>>, ui: &mut Ui) {
    // let height = height / 2.0;
    let crawling = crawler.lock().unwrap().crawling;
    ui.add_enabled_ui(crawling, |ui| {
        let prog = crawler.lock().unwrap().prog.round();
        let msg = if prog < 100.0 {
            RichText::new(format!("{} ... {:?}%", crawler.lock().unwrap().msg, prog))
        } else {
            RichText::new(format!(
                "Crawling completed\n{}",
                crawler.lock().unwrap().msg
            ))
        };
        ui.add_space(SPACE);
        ui.label(msg);
        ui.add_space(SPACE);
        if crawling {
            ui.spinner();
        }
        ui.add(ProgressBar::new(prog.round() / 100.0));
        ui.add_space(SPACE);
    });
}
fn ping_progress(ping: &Arc<Mutex<Ping>>, ui: &mut Ui) {
    let pinging = ping.lock().unwrap().pinging;
    ui.add_enabled_ui(pinging, |ui| {
        let prog = ping.lock().unwrap().prog.round();
        let msg = RichText::new(format!("{} ... {}%", ping.lock().unwrap().msg, prog));
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
}
fn last_node_button(
    width: f32,
    height: f32,
    selected_node: &mut RemoteNode,
    ping: &Arc<Mutex<Ping>>,
    ui: &mut Ui,
) {
    // make sense to use only if there is more than one found node.
    let ping_lock = ping.lock().unwrap();
    let found_multiple_nodes = ping_lock.nodes.len() > 1;
    ui.add_enabled_ui(found_multiple_nodes, |ui| {
        if ui
            .add_sized([width, height], Button::new("⬅ Last"))
            .on_hover_text(P2POOL_SELECT_LAST)
            .clicked()
        {
            match ping_lock.pinged {
                true => *selected_node = ping_lock.get_last_from_ping(selected_node),
                false => *selected_node = ping_lock.nodes.get_last(selected_node),
            }
        }
    })
    .response
    .on_disabled_hover_text(BUTTON_DISABLED_BY_EMPTY_LIST_NODES);
}
fn next_node_button(
    width: f32,
    height: f32,
    selected_node: &mut RemoteNode,
    ping: &Arc<Mutex<Ping>>,
    ui: &mut Ui,
) {
    // make sense to use only if there is more than one found node.
    let ping_lock = ping.lock().unwrap();
    let found_multiple_nodes = ping_lock.nodes.len() > 1;
    ui.add_enabled_ui(found_multiple_nodes, |ui| {
        if ui
            .add_sized([width, height], Button::new("Next ➡"))
            .on_hover_text(P2POOL_SELECT_NEXT)
            .clicked()
        {
            match ping_lock.pinged {
                true => *selected_node = ping_lock.get_next_from_ping(selected_node),
                false => *selected_node = ping_lock.nodes.get_next(selected_node),
            }
        }
    });
}
fn ping_button(ping: &Arc<Mutex<Ping>>, width: f32, height: f32, ui: &mut Ui) {
    // disable if no nodes were found or if we already are pinging
    let ping_lock = ping.lock().unwrap();
    let node_found = !ping_lock.nodes.is_empty();
    let pinging = ping_lock.pinging;
    ui.add_enabled_ui(node_found && !pinging, |ui| {
        if ui
            .add_sized([width, height], Button::new("Ping remote nodes"))
            .on_hover_text(P2POOL_PING)
            .clicked()
        {
            // drop(ping_lock);
            Ping::spawn_thread(ping);
        }
    })
    .response
    .on_disabled_hover_text(if node_found {
        BUTTON_DISABLED_BY_EMPTY_LIST_NODES
    } else {
        "Already pinging remote nodes"
    });
}
fn select_fastest_node_button(
    ping: &Arc<Mutex<Ping>>,
    selected_node: &mut RemoteNode,
    width: f32,
    height: f32,
    ui: &mut Ui,
) {
    // make sense to use only if there is more than one found node.
    let ping_lock = ping.lock().unwrap();
    let found_multiple_nodes = ping_lock.nodes.len() > 1;
    ui.add_enabled_ui(found_multiple_nodes, |ui| {
        if ui
            .add_sized([width, height], Button::new("Select fastest node"))
            .on_hover_text(P2POOL_SELECT_FASTEST)
            .clicked()
        {
            *selected_node = ping_lock
                .nodes
                .first()
                .expect(EXPECT_BUTTON_DISABLED)
                .clone();
        }
    })
    .response
    .on_disabled_hover_text(BUTTON_DISABLED_BY_EMPTY_LIST_NODES);
}
fn select_random_node_button(
    ping: &Arc<Mutex<Ping>>,
    selected_node: &mut RemoteNode,
    width: f32,
    height: f32,
    ui: &mut Ui,
) {
    // make sense to use only if there is more than one found node.
    let ping_lock = ping.lock().unwrap();
    let found_multiple_nodes = ping_lock.nodes.len() > 1;
    ui.add_enabled_ui(found_multiple_nodes, |ui| {
        if ui
            .add_sized([width, height], Button::new("Select random node"))
            .on_hover_text(P2POOL_SELECT_RANDOM)
            .clicked()
        {
            let node = ping_lock
                .nodes
                .get_random_same_ok()
                .expect(EXPECT_BUTTON_DISABLED)
                .clone();
            *selected_node = node;
        }
    })
    .response
    .on_disabled_hover_text(BUTTON_DISABLED_BY_EMPTY_LIST_NODES);
}
