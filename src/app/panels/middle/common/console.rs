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

use std::sync::{Arc, Mutex};

use egui::{Label, TextEdit, TextStyle, TextWrapMode, Ui};

use crate::{
    DARK_GRAY,
    helper::{Process, ProcessName},
    miscs::height_txt_before_button,
    regex::num_lines,
};

pub fn console(ui: &mut Ui, text: &str, console_height: &mut u32, process_name: ProcessName) {
    let nb_lines = num_lines(text);
    *console_height = egui::Resize::default()
        .id_salt(process_name.to_string())
        .default_height(*console_height as f32)
        .min_width(ui.available_width())
        .max_width(ui.available_width())
        .show(ui, |ui| {
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);
                ui.style_mut().override_text_style = Some(TextStyle::Small);
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_width(ui.available_width())
                    .max_height(ui.available_height())
                    .auto_shrink([false; 2])
                    // .show_viewport(ui, |ui, _| {
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Small),
                        nb_lines,
                        |ui, row_range| {
                            for i in row_range {
                                if let Some(line) = text.lines().nth(i) {
                                    ui.label(line);
                                }
                            }
                        },
                    );
            })
        })
        .response
        .rect
        .height() as u32;
}

// input args
pub fn input_args_field(
    ui: &mut Ui,
    buffer: &mut String,
    process: &Arc<Mutex<Process>>,
    hint: &str,
    hover: &str,
) {
    ui.style_mut().spacing.text_edit_width = ui.available_width();
    let response = ui
        .add(TextEdit::hint_text(TextEdit::singleline(buffer), hint))
        .on_hover_text(hover);
    // If the user pressed enter, dump buffer contents into the process STDIN
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        response.request_focus(); // Get focus back
        let buffer = std::mem::take(buffer); // Take buffer
        let mut process = process.lock().unwrap();
        if process.is_alive() {
            process.input.push(buffer);
        } // Push only if alive
    }
}

// Command arguments
pub fn start_options_field(ui: &mut Ui, arguments: &mut String, hint: &str, hover: &str) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.add_sized(
                [0.0, height_txt_before_button(ui, &TextStyle::Body)],
                Label::new("Command arguments:"),
            );
            ui.style_mut().spacing.text_edit_width = ui.available_width();
            ui.add(TextEdit::hint_text(TextEdit::singleline(arguments), hint))
                .on_hover_text(hover);
            arguments.truncate(1024);
        })
    });
    if !arguments.is_empty() {
        ui.disable();
    }
}
