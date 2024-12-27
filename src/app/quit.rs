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

use log::info;

use crate::errors::ErrorButtons;
use crate::errors::ErrorFerris;

use super::App;

impl App {
    pub(super) fn quit(&mut self, ctx: &egui::Context) {
        // If closing.
        // Used to be `eframe::App::on_close_event(&mut self) -> bool`.
        let close_signal = ctx.input(|input| {
            use egui::viewport::ViewportCommand;

            if !input.viewport().close_requested() {
                return None;
            }
            info!("quit");
            if self.state.gupax.auto.ask_before_quit {
                // If we're already on the [ask_before_quit] screen and
                // the user tried to exit again, exit.
                if self.error_state.quit_twice {
                    if self.state.gupax.auto.save_before_quit {
                        self.save_before_quit();
                    }
                    return Some(ViewportCommand::Close);
                }
                // Else, set the error
                self.error_state
                    .set("", ErrorFerris::Oops, ErrorButtons::StayQuit);
                self.error_state.quit_twice = true;
                Some(ViewportCommand::CancelClose)
            // Else, just quit.
            } else {
                if self.state.gupax.auto.save_before_quit {
                    self.save_before_quit();
                }
                Some(ViewportCommand::Close)
            }
        });
        // This will either:
        // 1. Cancel a close signal
        // 2. Close the program
        if let Some(cmd) = close_signal {
            ctx.send_viewport_cmd(cmd);
        }
    }
}
