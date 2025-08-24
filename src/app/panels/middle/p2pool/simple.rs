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

use crate::app::BackupNodes;
use crate::constants::*;
use crate::disk::state::P2pool;
use crate::helper::crawler::Crawler;
use egui::Ui;

impl P2pool {
    pub(super) fn simple(
        &mut self,
        ui: &mut Ui,
        crawler: &Arc<Mutex<Crawler>>,
        backup_hosts: BackupNodes,
    ) {
        ui.vertical_centered(|ui|{
            ui.add_space(SPACE);
            ui.checkbox(&mut self.local_node, P2POOL_USE_LOCAL_NODE_BUTTON).on_hover_text("If checked (recommended), p2pool will start trying to use the local node.\nThe local node can be started from or without Gupaxx, as long as it is p2pool capable.\nCheck the Node tab to start a local node.\n\nIf unchecked (default), p2pool will attempt to find and use a remote node by crawling the network.");
        });

        ui.add_space(SPACE * 2.0);
        // if checked, use only local node
        // if unchecked, enable button for crawling
        ui.add_enabled_ui(!self.local_node, |ui| {
            self.crawl_button(crawler, backup_hosts, ui);
        });
    }
}
