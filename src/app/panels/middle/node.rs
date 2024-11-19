use crate::{
    GUPAX_SELECT, NODE_API_BIND, NODE_API_PORT, NODE_ARGUMENTS, NODE_DB_DIR, NODE_DB_PATH_EMPTY,
    NODE_DNS_BLOCKLIST, NODE_DNS_CHECKPOINT, NODE_INPUT, NODE_PATH_OK, NODE_PRUNNING, NODE_URL,
    NODE_ZMQ_BIND, NODE_ZMQ_PORT,
};
use egui::{Color32, Label, RichText, Slider, TextEdit, TextStyle, Ui, Vec2};
use regex::Regex;
use std::sync::{Arc, Mutex};

use log::debug;

use crate::components::gupax::{FileType, FileWindow};
use crate::disk::state::{Gupax, Node};
use crate::helper::Process;
use crate::helper::node::PubNodeApi;
use crate::regex::{REGEXES, num_lines};
use crate::utils::constants::DARK_GRAY;
use crate::{GREEN, LIGHT_GRAY, P2POOL_IN, P2POOL_LOG, P2POOL_OUT, RED, SPACE};

impl Node {
    #[inline(always)] // called once
    pub fn show(
        &mut self,
        process: &Arc<Mutex<Process>>,
        api: &Arc<Mutex<PubNodeApi>>,
        buffer: &mut String,
        size: Vec2,
        file_window: &Arc<Mutex<FileWindow>>,
        ui: &mut egui::Ui,
    ) {
        let width = size.x;
        let height = size.y;
        let space_h = height / 48.0;
        let text_height = size.y / 25.0;
        let txt_description_width = size.x * 0.1;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(space_h);
                ui.style_mut().override_text_style = Some(TextStyle::Heading);
                ui.hyperlink_to("Monerod", NODE_URL);
                ui.style_mut().override_text_style = Some(TextStyle::Body);
                ui.add(Label::new("C++ Monero Node"));
                ui.add_space(space_h);
            });
            // console output for log
            debug!("Node Tab | Rendering [Console]");
            ui.group(|ui| {
                let text = &api.lock().unwrap().output;
                let nb_lines = num_lines(text);
                let height = size.y / 2.8;
                let width = (size.x - (space_h / 2.0)).max(0.0);
                egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(TextStyle::Small);
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_width(width)
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
            });
            //---------------------------------------------------------------------------------------------------- [Advanced] Console
            if !self.simple {
                ui.separator();
                let response = ui
                    .add_sized(
                        [width, text_height],
                        TextEdit::hint_text(
                            TextEdit::singleline(buffer),
                            r#"Commands: help, status, set_log <level>, diff"#,
                        ),
                    )
                    .on_hover_text(NODE_INPUT);
                // If the user pressed enter, dump buffer contents into the process STDIN
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    response.request_focus(); // Get focus back
                    let buffer = std::mem::take(buffer); // Take buffer
                    let mut process = process.lock().unwrap(); // Lock
                    if process.is_alive() {
                        process.input.push(buffer);
                    } // Push only if alive
                }

                //---------------------------------------------------------------------------------------------------- Arguments
                debug!("Node Tab | Rendering [Arguments]");
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [txt_description_width, text_height],
                            Label::new("Command arguments:"),
                        );
                        ui.add_sized(
                            [ui.available_width(), text_height],
                            TextEdit::hint_text(
                                TextEdit::singleline(&mut self.arguments),
                                r#"--zmq-pub tcp://127.0.0.1:18081"#,
                            ),
                        )
                        .on_hover_text(NODE_ARGUMENTS);
                        self.arguments.truncate(1024);
                    })
                });
                if !self.arguments.is_empty() {
                    ui.disable();
                }
                //---------------------------------------------------------------------------------------------------- Prunned checkbox
                ui.add_space(space_h);
                ui.style_mut().spacing.icon_width_inner = width / 45.0;
                ui.style_mut().spacing.icon_width = width / 35.0;
                ui.style_mut().spacing.icon_spacing = space_h;
                debug!("Node Tab | Rendering DNS  and Prunning buttons");
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.checkbox(&mut self.pruned, "Prunned")
                            .on_hover_text(NODE_PRUNNING);
                        ui.separator();
                        ui.checkbox(&mut self.dns_blocklist, "DNS blocklist")
                            .on_hover_text(NODE_DNS_BLOCKLIST);
                        ui.separator();
                        ui.checkbox(&mut self.disable_dns_checkpoint, "DNS checkpoint")
                            .on_hover_text(NODE_DNS_CHECKPOINT);
                    });
                });

                ui.add_space(space_h);
                // idea
                // need to warn the user if local firewall is blocking port
                // need to warn the user if NAT is blocking port
                // need to show local ip address
                // need to show public ip
                // text edit width is 4x bigger than description. Which makes half of the total width on screen less a space.

                // (width - (width - ui.available_width()) - (ui.spacing().item_spacing.x * 4.5))
                // / 2.0;
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            rpc_bind_field(self, ui, txt_description_width, text_height, width);
                            rpc_port_field(self, ui, txt_description_width, text_height, width);
                            ui.add_space(space_h);
                            zmq_bind_field(self, ui, txt_description_width, text_height, width);
                            zmq_port_field(self, ui, txt_description_width, text_height, width);
                        });
                    });

                    //---------------------------------------------------------------------------------------------------- In/Out peers
                    debug!("Node Tab | Rendering sliders elements");
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.style_mut().override_text_style = Some(TextStyle::Small);
                            ui.horizontal(|ui| {
                                // ui.label("Out peers [10-450]:");
                                ui.add_sized(
                                    [txt_description_width, text_height],
                                    Label::new("Out peers [2-450]:"),
                                );
                                // not sure what's the right calculation to make
                                ui.style_mut().spacing.slider_width = (ui.available_width()
                                    - ui.spacing().item_spacing.x * 4.0
                                    - ui.spacing().scroll.bar_width
                                    - (SPACE * 2.0))
                                    .max(0.0);
                                ui.add(Slider::new(&mut self.out_peers, 2..=450))
                                    .on_hover_text(P2POOL_OUT);
                                // ui.add_space(ui.available_width() - 4.0);
                            });
                            ui.horizontal(|ui| {
                                // ui.label("In peers  [10-450]:");
                                ui.add_sized(
                                    [txt_description_width, text_height],
                                    Label::new("In peers  [2-450]:"),
                                );
                                ui.style_mut().spacing.slider_width = (ui.available_width()
                                    - ui.spacing().item_spacing.x * 4.0
                                    - ui.spacing().scroll.bar_width
                                    - (SPACE * 2.0))
                                    .max(0.0);
                                ui.add(Slider::new(&mut self.in_peers, 2..=450))
                                    .on_hover_text(P2POOL_IN);
                            });
                            ui.horizontal(|ui| {
                                // ui.label("Log level  [ 0-4 ]:");
                                ui.add_sized(
                                    [txt_description_width, text_height],
                                    Label::new("Log level [ 0-4 ] :"),
                                );
                                ui.style_mut().spacing.slider_width = (ui.available_width()
                                    - ui.spacing().item_spacing.x * 4.0
                                    - ui.spacing().scroll.bar_width
                                    - (SPACE * 2.0))
                                    .max(0.0);
                                ui.add(Slider::new(&mut self.log_level, 0..=4))
                                    .on_hover_text(P2POOL_LOG);
                            });
                        });
                    });
                });
                //---------------------------------------------------------------------------------------------------- DB path
                ui.add_space(space_h);
                ui.group(|ui| {
                    path_db_field(self, ui, txt_description_width, text_height, file_window);
                });
                ui.add_space(space_h);
            }
        });
    }
}

fn rpc_bind_field(
    state: &mut Node,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    width: f32,
) {
    state_edit_field(
        &mut state.api_ip,
        ui,
        txt_description_width,
        text_height,
        width,
        "RPC BIND IP ",
        255,
        NODE_API_BIND,
        vec![&REGEXES.ipv4, &REGEXES.domain],
    );
}

fn rpc_port_field(
    state: &mut Node,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    width: f32,
) {
    state_edit_field(
        &mut state.api_port,
        ui,
        txt_description_width,
        text_height,
        width,
        "   RPC PORT ",
        5,
        NODE_API_PORT,
        vec![&REGEXES.port],
    );
}
fn zmq_bind_field(
    state: &mut Node,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    width: f32,
) {
    state_edit_field(
        &mut state.zmq_ip,
        ui,
        txt_description_width,
        text_height,
        width,
        "API BIND IP ",
        255,
        NODE_ZMQ_BIND,
        vec![&REGEXES.ipv4, &REGEXES.domain],
    );
}
fn zmq_port_field(
    state: &mut Node,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    width: f32,
) {
    state_edit_field(
        &mut state.zmq_port,
        ui,
        txt_description_width,
        text_height,
        width,
        "   ZMQ PORT ",
        5,
        NODE_ZMQ_PORT,
        vec![&REGEXES.port],
    );
}

fn path_db_field(
    state: &mut Node,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    file_window: &Arc<Mutex<FileWindow>>,
) {
    ui.horizontal(|ui| {
        let symbol;
        let color;
        let hover;
        if state.path_db.is_empty() {
            symbol = "➖";
            color = LIGHT_GRAY;
            hover = NODE_DB_PATH_EMPTY;
        } else if !Gupax::path_is_dir(&state.path_db) {
            symbol = "❌";
            color = RED;
            hover = NODE_DB_DIR;
        } else {
            symbol = "✔";
            color = GREEN;
            hover = NODE_PATH_OK;
        }
        let text = ["Node Database Directory ", symbol].concat();
        ui.add_sized(
            [txt_description_width, text_height],
            Label::new(RichText::new(text).color(color)),
        );
        ui.spacing_mut().text_edit_width =
            (ui.available_width() - (ui.spacing().item_spacing.x * 8.0) - SPACE * 2.0).max(0.0);
        let window_busy = file_window.lock().unwrap().thread;
        ui.add_enabled_ui(!window_busy, |ui| {
            if ui.button("Open").on_hover_text(GUPAX_SELECT).clicked() {
                Gupax::spawn_file_window_thread(file_window, FileType::NodeDB);
            }
            ui.text_edit_singleline(&mut state.path_db)
                .on_hover_text(hover);
        });
    });

    let mut guard = file_window.lock().unwrap();
    if guard.picked_nodedb {
        state.path_db.clone_from(&guard.nodedb_path);
        guard.picked_nodedb = false;
    }
}
#[allow(clippy::too_many_arguments)]
fn state_edit_field(
    state_field: &mut String,
    ui: &mut Ui,
    txt_description_width: f32,
    text_height: f32,
    width: f32,
    description: &str,
    max_ch: u8,
    help_msg: &str,
    validations: Vec<&Regex>,
) {
    ui.horizontal(|ui| {
        let color;
        let symbol;
        let mut input_validated = true;
        let len;
        let inside_space;
        for v in validations {
            if !v.is_match(state_field) {
                input_validated = false;
            }
        }
        if state_field.is_empty() {
            symbol = "➖";
            color = Color32::LIGHT_GRAY;
        } else if input_validated {
            symbol = "✔";
            color = Color32::from_rgb(100, 230, 100);
        } else {
            symbol = "❌";
            color = Color32::from_rgb(230, 50, 50);
        }
        match max_ch {
            x if x >= 100 => {
                len = format!("{:03}", state_field.len());
                inside_space = "";
            }
            10..99 => {
                len = format!("{:02}", state_field.len());
                inside_space = " ";
            }
            _ => {
                len = format!("{}", state_field.len());
                inside_space = "  ";
            }
        }
        let text = format!(
            "{}[{}{}/{}{}]{}",
            description, inside_space, len, max_ch, inside_space, symbol
        );
        ui.add_sized(
            [txt_description_width, text_height],
            Label::new(RichText::new(text).color(color)),
        );
        // allocate the size to leave half of the total width free.
        ui.spacing_mut().text_edit_width = ((width / 2.0)
            - (width - ui.available_width() - ui.spacing().scroll.bar_width)
            - ui.spacing().item_spacing.x * 2.5)
            .max(0.0);
        ui.text_edit_singleline(state_field).on_hover_text(help_msg);
        state_field.truncate(max_ch.into());
    });
}
