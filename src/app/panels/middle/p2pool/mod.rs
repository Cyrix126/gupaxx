use crate::app::panels::middle::common::console::{console, input_args_field, start_options_field};
use crate::disk::state::{P2pool, StartOptionsMode, State};
use crate::helper::p2pool::PubP2poolApi;
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
use crate::{components::node::*, constants::*, helper::*};
use log::*;

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::common::header_tab::header_tab;
use super::common::list_poolnode::PoolNode;

mod advanced;
mod simple;

impl P2pool {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        node_vec: &mut Vec<(String, PoolNode)>,
        _og: &Arc<Mutex<State>>,
        ping: &Arc<Mutex<Ping>>,
        process: &Arc<Mutex<Process>>,
        api: &Arc<Mutex<PubP2poolApi>>,
        buffer: &mut String,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        backup_nodes: Option<Vec<PoolNode>>,
        path: &Path,
        local_node_zmq_port: u16,
        local_node_rpc_port: u16,
    ) {
        //---------------------------------------------------------------------------------------------------- [Simple] Console
        // debug!("P2Pool Tab | Rendering [Console]");
        let mut api_lock = api.lock().unwrap();
        // let mut prefer_local_node = api.lock().unwrap().prefer_local_node;
        header_tab(
            ui,
            None,
            &[("P2Pool", P2POOL_URL, "")],
            Some("Decentralized pool for Monero mining"),
            true,
        );
        egui::ScrollArea::vertical().show(ui, |ui| {
            let text = &api_lock.output;
            ui.group(|ui| {
                console(ui, text, &mut self.console_height, ProcessName::P2pool);
                if !self.simple {
                    ui.separator();
                    input_args_field(
                        ui,
                        buffer,
                        process,
                        r#"Type a command (e.g "help" or "status") and press Enter"#,
                        P2POOL_INPUT,
                    );
                }
            });
            if !self.simple {
                let default_args_simple = self.start_options(
                    path,
                    &backup_nodes,
                    StartOptionsMode::Simple,
                    local_node_zmq_port,
                    local_node_rpc_port,
                );
                let default_args_advanced = self.start_options(
                    path,
                    &backup_nodes,
                    StartOptionsMode::Advanced,
                    local_node_zmq_port,
                    local_node_rpc_port,
                );
                start_options_field(
                    ui,
                    &mut self.arguments,
                    &default_args_simple,
                    &default_args_advanced,
                    Self::process_name().start_options_hint(),
                    START_OPTIONS_HOVER,
                );
            }
            debug!("P2Pool Tab | Rendering [Address]");
            crate::app::panels::middle::common::state_edit_field::monero_address_field(
                &mut self.address,
                ui,
                P2POOL_ADDRESS,
            );

            if self.simple {
                self.simple(ui, ping, &mut api_lock);
            } else {
                if !self.arguments.is_empty() {
                    ui.disable();
                }
                self.advanced(ui, node_vec);
            }
        });
    }
}
