use std::sync::{Arc, Mutex};

use egui::TextStyle::Name;
use egui::{vec2, Hyperlink, Image, Layout, RichText, TextEdit, Ui, Vec2};
use log::debug;

use crate::helper::xvb::PubXvbApi;
use crate::utils::constants::{
    GREEN, LIGHT_GRAY, ORANGE, RED, XVB_DONATED_1H_FIELD, XVB_DONATED_24H_FIELD, XVB_FAILURE_FIELD,
    XVB_HELP, XVB_HERO_SELECT, XVB_TOKEN_FIELD, XVB_TOKEN_LEN, XVB_URL_RULES,
};
use crate::utils::macros::lock;
use crate::utils::regex::Regexes;
use crate::{
    constants::{BYTES_XVB, SPACE},
    utils::constants::{DARK_GRAY, XVB_URL},
};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    pub fn show(
        &mut self,
        size: Vec2,
        address: &str,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        api: &Arc<Mutex<PubXvbApi>>,
        xvb_is_alive: bool,
    ) {
        ui.reset_style();
        let website_height = size.y / 10.0;
        // let width = size.x - SPACE;
        // let height = size.y - SPACE;
        let width = size.x;
        // logo and website link
        ui.vertical_centered(|ui| {
            ui.add_sized(
                [width, website_height],
                Image::from_bytes("bytes:/xvb.png", BYTES_XVB),
            );
            ui.add_sized(
                [width / 8.0, website_height],
                Hyperlink::from_label_and_url("XMRvsBeast", XVB_URL),
            );
        });
        // console output for log
        debug!("XvB Tab | Rendering [Console]");
        ui.group(|ui| {
            let height = size.y / 2.8;
            let width = size.x - SPACE;
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
        });
        // input token
        let len_token = format!("{}", self.token.len());
        let (text, color) = if self.token.is_empty() {
            (
                format!("{} [{}/{}] ➖", XVB_TOKEN_FIELD, len_token, XVB_TOKEN_LEN),
                LIGHT_GRAY,
            )
        } else if self.token.parse::<u32>().is_ok() && self.token.len() < XVB_TOKEN_LEN {
            (
                format!("{} [{}/{}]", XVB_TOKEN_FIELD, len_token, XVB_TOKEN_LEN),
                GREEN,
            )
        } else if self.token.parse::<u32>().is_ok() && self.token.len() == XVB_TOKEN_LEN {
            (format!("{} ✔", XVB_TOKEN_FIELD), GREEN)
        } else {
            (
                format!("{} [{}/{}] ❌", XVB_TOKEN_FIELD, len_token, XVB_TOKEN_LEN),
                RED,
            )
        };
        // let width = width - SPACE;
        // ui.spacing_mut().text_edit_width = (width) - (SPACE * 3.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // why does this group is not centered into the parent group ?
                ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.group(|ui| {
                        ui.colored_label(color, text);
                        // ui.add_sized(
                        //     [width / 8.0, text_edit],
                        //     Label::new(RichText::new(text).color(color)),
                        // );
                        ui.add(
                            TextEdit::singleline(&mut self.token)
                                .char_limit(XVB_TOKEN_LEN)
                                .desired_width(width / 8.0)
                                .vertical_align(egui::Align::Center),
                        );
                    })
                    .response
                    .on_hover_text_at_pointer(XVB_HELP);
                    // hero option
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(width / 24.0);
                        ui.checkbox(&mut self.hero, "Hero")
                            .on_hover_text(XVB_HERO_SELECT);
                    })
                });
            })
        });
        // need to warn the user if no address is set in p2pool tab
        if !Regexes::addr_ok(address) {
            debug!("XvB Tab | Rendering warning text");
            ui.label(RichText::new("You don't have any payout address set in the P2pool Tab !\nXvB process needs one to function properly.")
                        .color(ORANGE));
        }
        // private stats
        let priv_stats = &lock!(api).stats_priv;
        ui.set_enabled(xvb_is_alive);
        // ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            // widget takes a third less space for two separator.
            let width_stat =
                (ui.available_width() / 3.0) - (12.0 + ui.style().spacing.item_spacing.x) / 3.0;
            // 0.0 means minimum
            let height_stat = 0.0;
            let size_stat = vec2(width_stat, height_stat);
            ui.add_sized(size_stat, |ui: &mut Ui| {
                ui.group(|ui| {
                    let size_stat = vec2(
                        ui.available_width(),
                        0.0, // + ui.spacing().item_spacing.y,
                    );
                    ui.add_sized(size_stat, |ui: &mut Ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(XVB_FAILURE_FIELD);
                            ui.label(priv_stats.fails.to_string());
                        })
                        .response
                    });
                    ui.separator();
                    ui.add_sized(size_stat, |ui: &mut Ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(XVB_DONATED_1H_FIELD);
                            ui.label(priv_stats.donor_1hr_avg.to_string());
                        })
                        .response
                    });
                    ui.separator();
                    ui.add_sized(size_stat, |ui: &mut Ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(XVB_DONATED_24H_FIELD);
                            ui.label(priv_stats.donor_24hr_avg.to_string());
                        })
                        .response
                    });
                })
                .response
            });
        });
        // Rules link help
        ui.add_space(ui.available_height() / 4.0);
        ui.vertical_centered(|ui| {
            ui.hyperlink_to("Rules", XVB_URL_RULES)
                .on_hover_text("Click here to read the rules and understand how the raffle works.");
        });
    }
}
