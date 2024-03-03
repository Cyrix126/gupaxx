use crate::disk::node::Node;
use crate::disk::state::{P2pool, State};
use crate::helper::p2pool::PubP2poolApi;
// Gupax - GUI Uniting P2Pool And XMRig
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
use crate::{components::node::*, constants::*, helper::*, macros::*, utils::regex::Regexes};
use egui::{Color32, Label, RichText, TextEdit, TextStyle::*};
use log::*;

use std::sync::{Arc, Mutex};

mod advanced;
mod simple;

impl P2pool {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        node_vec: &mut Vec<(String, Node)>,
        _og: &Arc<Mutex<State>>,
        ping: &Arc<Mutex<Ping>>,
        process: &Arc<Mutex<Process>>,
        api: &Arc<Mutex<PubP2poolApi>>,
        buffer: &mut String,
        width: f32,
        height: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        let text_edit = height / 25.0;
        //---------------------------------------------------------------------------------------------------- [Simple] Console
        debug!("P2Pool Tab | Rendering [Console]");
        ui.group(|ui| {
            if self.simple {
                let height = height / 2.8;
                let width = width - SPACE;
                egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_width(width)
                        .max_height(height)
                        .auto_shrink([false; 2])
                        .show_viewport(ui, |ui, _| {
                            ui.add_sized(
                                [width, height],
                                TextEdit::multiline(&mut lock!(api).output.as_str()),
                            );
                        });
                });
            //---------------------------------------------------------------------------------------------------- [Advanced] Console
            } else {
                let height = height / 2.8;
                let width = width - SPACE;
                egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_width(width)
                        .max_height(height)
                        .auto_shrink([false; 2])
                        .show_viewport(ui, |ui, _| {
                            ui.add_sized(
                                [width, height],
                                TextEdit::multiline(&mut lock!(api).output.as_str()),
                            );
                        });
                });
                ui.separator();
                let response = ui
                    .add_sized(
                        [width, text_edit],
                        TextEdit::hint_text(
                            TextEdit::singleline(buffer),
                            r#"Type a command (e.g "help" or "status") and press Enter"#,
                        ),
                    )
                    .on_hover_text(P2POOL_INPUT);
                // If the user pressed enter, dump buffer contents into the process STDIN
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    response.request_focus(); // Get focus back
                    let buffer = std::mem::take(buffer); // Take buffer
                    let mut process = lock!(process); // Lock
                    if process.is_alive() {
                        process.input.push(buffer);
                    } // Push only if alive
                }
            }
        });

        //---------------------------------------------------------------------------------------------------- Args
        if !self.simple {
            debug!("P2Pool Tab | Rendering [Arguments]");
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let width = (width / 10.0) - SPACE;
                    ui.add_sized([width, text_edit], Label::new("Command arguments:"));
                    ui.add_sized(
                        [ui.available_width(), text_edit],
                        TextEdit::hint_text(
                            TextEdit::singleline(&mut self.arguments),
                            r#"--wallet <...> --host <...>"#,
                        ),
                    )
                    .on_hover_text(P2POOL_ARGUMENTS);
                    self.arguments.truncate(1024);
                })
            });
            ui.set_enabled(self.arguments.is_empty());
        }

        //---------------------------------------------------------------------------------------------------- Address
        debug!("P2Pool Tab | Rendering [Address]");
        ui.group(|ui| {
            let width = width - SPACE;
            ui.spacing_mut().text_edit_width = (width) - (SPACE * 3.0);
            let text;
            let color;
            let len = format!("{:02}", self.address.len());
            if self.address.is_empty() {
                text = format!("Monero Address [{}/95] ➖", len);
                color = Color32::LIGHT_GRAY;
            } else if Regexes::addr_ok(&self.address) {
                text = format!("Monero Address [{}/95] ✔", len);
                color = Color32::from_rgb(100, 230, 100);
            } else {
                text = format!("Monero Address [{}/95] ❌", len);
                color = Color32::from_rgb(230, 50, 50);
            }
            ui.add_sized(
                [width, text_edit],
                Label::new(RichText::new(text).color(color)),
            );
            ui.add_sized(
                [width, text_edit],
                TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4..."),
            )
            .on_hover_text(P2POOL_ADDRESS);
            self.address.truncate(95);
        });

        let height = ui.available_height();
        if self.simple {
            //---------------------------------------------------------------------------------------------------- Simple
            self.simple(ui, width, height, ping);
        //---------------------------------------------------------------------------------------------------- Advanced
        } else {
            self.advanced(ui, width, height, text_edit, node_vec);
        }
    }
}
