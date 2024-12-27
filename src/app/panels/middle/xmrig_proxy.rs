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

use egui::{Checkbox, TextStyle, Ui, vec2};
use std::sync::{Arc, Mutex};

use log::debug;

use crate::app::panels::middle::common::console::{console, input_args_field, start_options_field};
use crate::app::panels::middle::common::header_tab::header_tab;
use crate::app::panels::middle::common::list_poolnode::list_poolnode;
use crate::disk::state::XmrigProxy;
use crate::helper::Process;
use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::miscs::height_txt_before_button;
use crate::regex::REGEXES;
use crate::{
    SPACE, XMRIG_API_IP, XMRIG_API_PORT, XMRIG_IP, XMRIG_KEEPALIVE, XMRIG_NAME, XMRIG_PORT,
    XMRIG_PROXY_ARGUMENTS, XMRIG_PROXY_INPUT, XMRIG_PROXY_REDIRECT, XMRIG_PROXY_URL, XMRIG_RIG,
    XMRIG_TLS,
};

use super::common::list_poolnode::PoolNode;
use super::common::state_edit_field::StateTextEdit;

impl XmrigProxy {
    #[inline(always)] // called once
    pub fn show(
        &mut self,
        process: &Arc<Mutex<Process>>,
        pool_vec: &mut Vec<(String, PoolNode)>,
        api: &Arc<Mutex<PubXmrigProxyApi>>,
        buffer: &mut String,
        ui: &mut egui::Ui,
    ) {
        header_tab(
            ui,
            None,
            &[("XMRig-Proxy", XMRIG_PROXY_URL, "")],
            Some("High performant proxy for your miners"),
            true,
        );
        // console output for log
        debug!("Xmrig-Proxy Tab | Rendering [Console]");
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                let text = &api.lock().unwrap().output;
                console(ui, text);
                //---------------------------------------------------------------------------------------------------- [Advanced] Console
                if !self.simple {
                    ui.separator();
                    input_args_field(
                        ui,
                        buffer,
                        process,
                        r#"Commands: [h]ashrate, [c]onnections, [v]erbose, [w]orkers"#,
                        XMRIG_PROXY_INPUT,
                    );
                }
            });
            if !self.simple {
                //---------------------------------------------------------------------------------------------------- Arguments
                debug!("XMRig-Proxy Tab | Rendering [Arguments]");
                ui.horizontal(|ui| {
                    start_options_field(
                        ui,
                        &mut self.arguments,
                        r#"--url <...> --user <...> --config <...>"#,
                        XMRIG_PROXY_ARGUMENTS,
                    );
                });
                if !self.arguments.is_empty() {
                    ui.disable();
                }
                ui.add_space(SPACE);
                // ui.style_mut().spacing.icon_width_inner = width / 45.0;
                // ui.style_mut().spacing.icon_width = width / 35.0;
                // ui.style_mut().spacing.icon_spacing = space_h;
                ui.checkbox(
                    &mut self.redirect_local_xmrig,
                    "Auto Redirect local Xmrig to Xmrig-Proxy",
                )
                .on_hover_text(XMRIG_PROXY_REDIRECT);

                // idea
                // need to warn the user if local firewall is blocking port
                // need to warn the user if NAT is blocking port
                // need to show local ip address
                // need to show public ip

                debug!("XMRig-Proxy Tab | Rendering [Pool List] elements");
                // let width = ui.available_width() - 10.0;
                let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
                // [Pool IP/Port]
                egui::ScrollArea::horizontal()
                    .id_salt("proxy_horizontal")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.group(|ui| {
                                // let width = width / 10.0;
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
                                    if !self.rig_field(ui) {
                                        incorrect_input = false;
                                    }
                                });

                                ui.vertical(|ui| {
                                    list_poolnode(
                                        ui,
                                        &mut (
                                            &mut self.name,
                                            &mut self.ip,
                                            &mut self.port,
                                            &mut self.rig,
                                        ),
                                        &mut self.selected_pool,
                                        pool_vec,
                                        incorrect_input,
                                    );
                                });
                            });
                        });
                        ui.add_space(5.0);

                        debug!("XMRig-Proxy Tab | Rendering [API] TextEdits");
                        // [HTTP API IP/Port]
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    // HTTP API
                                    self.api_ip_field(ui);
                                    self.api_port_field(ui);
                                });

                                ui.separator();

                                debug!("XMRig-Proxy Tab | Rendering [TLS/Keepalive] buttons");
                                ui.vertical(|ui| {
                                    // TLS/Keepalive
                                    ui.horizontal(|ui| {
                                        let width = (ui.available_width() / 2.0) - 11.0;
                                        let height =
                                            height_txt_before_button(ui, &TextStyle::Button) * 2.0;
                                        let size = vec2(width, height);
                                        ui.add_sized(
                                            size,
                                            Checkbox::new(&mut self.tls, "TLS Connection"),
                                        )
                                        .on_hover_text(XMRIG_TLS);
                                        ui.separator();
                                        ui.add_sized(
                                            size,
                                            Checkbox::new(&mut self.keepalive, "Keepalive"),
                                        )
                                        .on_hover_text(XMRIG_KEEPALIVE);
                                    });
                                });
                            });
                        });
                    });
            }
        });
    }
    fn name_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   Name     ")
            .max_ch(30)
            .help_msg(XMRIG_NAME)
            .validations(&[|x| REGEXES.name.is_match(x)])
            .build(ui, &mut self.name)
    }
    fn rpc_port_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   RPC PORT ")
            .max_ch(5)
            .help_msg(XMRIG_PORT)
            .validations(&[|x| REGEXES.port.is_match(x)])
            .build(ui, &mut self.port)
    }
    fn ip_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description("   IP       ")
            .max_ch(255)
            .help_msg(XMRIG_IP)
            .validations(&[|x| REGEXES.ipv4.is_match(x) || REGEXES.domain.is_match(x)])
            .build(ui, &mut self.ip)
    }
    fn rig_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description(" RIG ID   ")
            .max_ch(30)
            .help_msg(XMRIG_RIG)
            .build(ui, &mut self.rig)
    }
    fn api_ip_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description(" API IP   ")
            .max_ch(255)
            .help_msg(XMRIG_API_IP)
            .validations(&[|x| REGEXES.ipv4.is_match(x) || REGEXES.domain.is_match(x)])
            .build(ui, &mut self.api_ip)
    }
    fn api_port_field(&mut self, ui: &mut Ui) -> bool {
        StateTextEdit::new(ui)
            .description(" API PORT ")
            .max_ch(5)
            .help_msg(XMRIG_API_PORT)
            .validations(&[|x| REGEXES.port.is_match(x)])
            .build(ui, &mut self.api_port)
    }
}
