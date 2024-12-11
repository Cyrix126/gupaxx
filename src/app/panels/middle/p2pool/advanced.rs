use crate::app::panels::middle::common::list_poolnode::{PoolNode, list_poolnode};
use crate::app::panels::middle::common::state_edit_field::{StateTextEdit, slider_state_field};
use crate::miscs::height_txt_before_button;
use crate::{disk::state::P2pool, utils::regex::REGEXES};

use crate::constants::*;
use egui::{Checkbox, SelectableLabel, Ui};
use log::*;

impl P2pool {
    pub(super) fn advanced(&mut self, ui: &mut Ui, node_vec: &mut Vec<(String, PoolNode)>) {
        // let height = size.y / 16.0;
        // let space_h = size.y / 128.0;
        debug!("P2Pool Tab | Rendering [Node List] elements");
        let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
        // [Monero node IP/RPC/ZMQ]
        ui.horizontal(|ui| {
            ui.group(|ui| {
                // let width = size.x/10.0;
                ui.vertical(|ui| {
                    if !self.name_field(ui) {
                        incorrect_input = false;
                    }
                    if !self.ip_field(ui) {
                        incorrect_input = false;
                    }
                    if !self.rpc_port_field(ui) {
                        incorrect_input = false;
                    }
                    if !self.zmq_port_field(ui) {
                        incorrect_input = false;
                    }
                });
                list_poolnode(
                    ui,
                    &mut (&mut self.name, &mut self.ip, &mut self.rpc, &mut self.zmq),
                    &mut self.selected_node,
                    node_vec,
                    incorrect_input,
                );
            });
        });
        // ui.add_space(space_h);

        debug!("P2Pool Tab | Rendering [Main/Mini/Peers/Log] elements");
        // [Main/Mini]
        ui.horizontal(|ui| {
            // let height = height / 4.0;
            ui.group(|ui| {
                ui.vertical(|ui| {
                    let height = height_txt_before_button(ui, &egui::TextStyle::Button) * 1.9;
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() / 4.0) - SPACE;
                        if ui
                            // if ui.add_sized(, )
                            // .selectable_label(!self.mini, "P2Pool Main")
                            .add_sized(
                                [width, height],
                                SelectableLabel::new(!self.mini, "P2Pool Main"),
                            )
                            .on_hover_text(P2POOL_MAIN)
                            .clicked()
                        {
                            self.mini = false;
                        }
                        if ui
                            // .selectable_label(!self.mini, "P2Pool Mini")
                            // if ui
                            .add_sized(
                                [width, height],
                                SelectableLabel::new(self.mini, "P2Pool Mini"),
                            )
                            .on_hover_text(P2POOL_MINI)
                            .clicked()
                        {
                            self.mini = true;
                        }
                    });
                    debug!("P2Pool Tab | Rendering Backup host button");
                    ui.group(|ui| {
                        // [Backup host]
                        ui.add_sized(
                            [(ui.available_width() / 2.0) - (SPACE * 2.0), height],
                            Checkbox::new(&mut self.backup_host, "Backup host"),
                        )
                        // ui.checkbox(&mut self.backup_host, "Backup host")
                        .on_hover_text(P2POOL_BACKUP_HOST_ADVANCED);
                    });
                });
            });
            // [Out/In Peers] + [Log Level]
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(SPACE);
                    slider_state_field(
                        ui,
                        "Out peers [2-450]:",
                        P2POOL_OUT,
                        &mut self.out_peers,
                        2..=450,
                    );
                    ui.add_space(SPACE);
                    slider_state_field(
                        ui,
                        "In peers  [2-450]:",
                        P2POOL_IN,
                        &mut self.in_peers,
                        2..=450,
                    );
                    ui.add_space(SPACE);
                    slider_state_field(
                        ui,
                        "Log level [ 0-6 ]:",
                        P2POOL_LOG,
                        &mut self.log_level,
                        0..=6,
                    );
                })
            });
        });
    }
    fn name_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   Name     ")
            .max_ch(30)
            .help_msg(P2POOL_NAME)
            .validations(&[|x| REGEXES.name.is_match(x)])
            .build(ui, &mut self.name)
    }
    fn rpc_port_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   RPC PORT ")
            .max_ch(5)
            .help_msg(P2POOL_RPC_PORT)
            .validations(&[|x| REGEXES.port.is_match(x)])
            .build(ui, &mut self.rpc)
    }
    fn zmq_port_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   ZMQ PORT ")
            .max_ch(5)
            .help_msg(P2POOL_ZMQ_PORT)
            .validations(&[|x| REGEXES.port.is_match(x)])
            .build(ui, &mut self.zmq)
    }
    fn ip_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   IP       ")
            .max_ch(255)
            .help_msg(P2POOL_NODE_IP)
            .validations(&[|x| REGEXES.ipv4.is_match(x), |x| REGEXES.domain.is_match(x)])
            .build(ui, &mut self.ip)
    }
}
