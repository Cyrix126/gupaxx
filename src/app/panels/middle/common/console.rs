use std::sync::{Arc, Mutex};

use egui::{Label, TextEdit, TextStyle, Ui};

use crate::{DARK_GRAY, helper::Process, miscs::height_txt_before_button, regex::num_lines};

pub fn console(ui: &mut Ui, text: &str) {
    let nb_lines = num_lines(text);
    let height = ui.available_height() / 2.8;
    egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
        ui.style_mut().override_text_style = Some(TextStyle::Small);
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_width(ui.available_width())
            .max_height(height)
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
    });
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
