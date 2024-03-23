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
    pub thread: bool,        // Is there already a FileWindow thread?
    pub picked_p2pool: bool, // Did the user pick a path for p2pool?
    pub picked_xmrig: bool,  // Did the user pick a path for xmrig?
    pub p2pool_path: String, // The picked p2pool path
    pub xmrig_path: String,  // The picked p2pool path
}

impl FileWindow {
    pub fn new() -> Arc<Mutex<Self>> {
        arc_mut!(Self {
            thread: false,
            picked_p2pool: false,
            picked_xmrig: false,
            p2pool_path: String::new(),
            xmrig_path: String::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    P2pool,
    Xmrig,
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

    #[cold]
    #[inline(never)]
    pub fn spawn_file_window_thread(file_window: &Arc<Mutex<FileWindow>>, file_type: FileType) {
        use FileType::*;
        let name = match file_type {
            P2pool => "P2Pool",
            Xmrig => "XMRig",
        };
        let file_window = file_window.clone();
        lock!(file_window).thread = true;
        thread::spawn(move || {
            match rfd::FileDialog::new()
                .set_title(format!("Select {} Binary for Gupaxx", name))
                .pick_file()
            {
                Some(path) => {
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
                    };
                }
                None => info!("Gupaxx | No path selected for {}", name),
            };
            lock!(file_window).thread = false;
        });
    }
}
