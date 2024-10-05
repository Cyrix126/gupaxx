// Gupaxx - GUI Uniting P2Pool And XMRig
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

use crate::{
    disk::state::*,
    utils::macros::{arc_mut, lock},
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread,
};

//---------------------------------------------------------------------------------------------------- FileWindow
// Struct for writing/reading the path state.
// The opened file picker is started in a new
// thread so main() needs to be in sync.
pub struct FileWindow {
    pub thread: bool,             // Is there already a FileWindow thread?
    pub picked_p2pool: bool,      // Did the user pick a path for p2pool?
    pub picked_xmrig: bool,       // Did the user pick a path for xmrig?
    pub picked_xp: bool,          // Did the user pick a path for xmrig-proxy?
    pub picked_node: bool,        // Did the user pick a path for node?
    pub picked_nodedb: bool,      // Did the user pick a path for node?
    pub p2pool_path: String,      // The picked p2pool path
    pub node_path: String,        // The picked node path
    pub nodedb_path: String,      // The picked node path
    pub xmrig_path: String,       // The picked xmrig path
    pub xmrig_proxy_path: String, // The picked xmrig-proxy path
}

impl FileWindow {
    pub fn new() -> Arc<Mutex<Self>> {
        arc_mut!(Self {
            thread: false,
            picked_p2pool: false,
            picked_xmrig: false,
            picked_xp: false,
            picked_node: false,
            picked_nodedb: false,
            p2pool_path: String::new(),
            node_path: String::new(),
            nodedb_path: String::new(),
            xmrig_path: String::new(),
            xmrig_proxy_path: String::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    P2pool,
    Xmrig,
    XmrigProxy,
    Node,
    NodeDB,
}

//---------------------------------------------------------------------------------------------------- Ratio Lock
// Enum for the lock ratio in the advanced tab.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum Ratio {
    Width,
    Height,
    None,
}

//---------------------------------------------------------------------------------------------------- Gupaxx
impl Gupax {
    // Checks if a path is a valid path to a file.
    pub fn path_is_file(path: &str) -> bool {
        let path = path.to_string();
        match crate::disk::into_absolute_path(path) {
            Ok(path) => path.is_file(),
            _ => false,
        }
    }
    // Checks if a path is a valid path to a directory.
    pub fn path_is_dir(path: &str) -> bool {
        let path = path.to_string();
        match crate::disk::into_absolute_path(path) {
            Ok(path) => path.is_dir(),
            _ => false,
        }
    }

    #[cold]
    #[inline(never)]
    pub fn spawn_file_window_thread(file_window: &Arc<Mutex<FileWindow>>, file_type: FileType) {
        use FileType::*;
        let name = match file_type {
            P2pool => "P2Pool",
            Xmrig => "XMRig",
            XmrigProxy => "XMRigProxy",
            Node => "Node",
            NodeDB => "Node DB",
        };
        let file_window = file_window.clone();
        lock!(file_window).thread = true;
        thread::spawn(move || {
            let path = match file_type {
                NodeDB => rfd::FileDialog::new()
                    .set_title("Select a directory for the DB of your Node")
                    .pick_folder(),
                _ => rfd::FileDialog::new()
                    .set_title(format!("Select {} Binary for Gupaxx", name))
                    .pick_file(),
            };
            if let Some(path) = path {
                info!("Gupaxx | Path selected for {} ... {}", name, path.display());
                match file_type {
                    P2pool => {
                        lock!(file_window).p2pool_path = path.display().to_string();
                        lock!(file_window).picked_p2pool = true;
                    }
                    Xmrig => {
                        lock!(file_window).xmrig_path = path.display().to_string();
                        lock!(file_window).picked_xmrig = true;
                    }
                    XmrigProxy => {
                        lock!(file_window).xmrig_proxy_path = path.display().to_string();
                        lock!(file_window).picked_xp = true;
                    }
                    Node => {
                        lock!(file_window).node_path = path.display().to_string();
                        lock!(file_window).picked_node = true;
                    }
                    NodeDB => {
                        lock!(file_window).nodedb_path = path.display().to_string();
                        lock!(file_window).picked_nodedb = true;
                    }
                };
            } else {
                info!("Gupaxx | No path selected for {}", name);
            }

            lock!(file_window).thread = false;
        });
    }
}
