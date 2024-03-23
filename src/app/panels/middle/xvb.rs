use std::sync::{Arc, Mutex};

use egui::TextStyle::{self, Name};
use egui::{vec2, Image, RichText, TextEdit, Ui, Vec2};
use log::debug;
use readable::num::Float;

use crate::helper::xvb::PubXvbApi;
use crate::regex::num_lines;
use crate::utils::constants::{
    GREEN, LIGHT_GRAY, ORANGE, RED, XVB_DONATED_1H_FIELD, XVB_DONATED_24H_FIELD, XVB_FAILURE_FIELD,
    XVB_HELP, XVB_HERO_SELECT, XVB_ROUND_TYPE_FIELD, XVB_TOKEN_FIELD, XVB_TOKEN_LEN, XVB_URL_RULES,
    XVB_WINNER_FIELD,
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
        private_stats: bool,
    ) {
        let website_height = size.y / 10.0;
        let width = size.x;
        let height = size.y;
        let space_h = height / 48.0;
        // logo and website link
        ui.vertical_centered(|ui| {
            ui.add_sized(
                [width, website_height],
                Image::from_bytes("bytes:/xvb.png", BYTES_XVB),
            );
            ui.style_mut().override_text_style = Some(TextStyle::Heading);
            ui.add_space(space_h);
            ui.hyperlink_to("XMRvsBeast", XVB_URL);
            ui.add_space(space_h);
        });
        // console output for log
        debug!("XvB Tab | Rendering [Console]");
        ui.group(|ui| {
            let text = &lock!(api).output;
            let nb_lines = num_lines(text);
            let height = size.y / 2.8;
            let width = size.x - (space_h / 2.0);
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_width(width)
                    .max_height(height)
                    .auto_shrink([false; 2])
                    // .show_viewport(ui, |ui, _| {
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Name("MonospaceSmall".into())),
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
        ui.add_space(space_h);
        ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.colored_label(color, text);
                    ui.add(
                        TextEdit::singleline(&mut self.token)
                            .char_limit(9)
                            .desired_width(ui.text_style_height(&TextStyle::Body) * 9.0)
                            .vertical_align(egui::Align::Center),
                        )
            })
            .response
            .on_hover_text_at_pointer(XVB_HELP);
            ui.add_space(height / 48.0);
    ui.style_mut().spacing.icon_width_inner = width / 45.0;
    ui.style_mut().spacing.icon_width = width / 35.0;
    ui.style_mut().spacing.icon_spacing = space_h;
            ui.checkbox(&mut self.hero, "Hero Mode").on_hover_text(XVB_HERO_SELECT);
// need to warn the user if no address is set in p2pool tab
        if !Regexes::addr_ok(address) {
            ui.add_space(width / 16.0);
            debug!("XvB Tab | Rendering warning text");
                ui.horizontal_wrapped(|ui|{
            ui.label(RichText::new("You don't have any payout address set in the P2pool Tab ! XvB process needs one to function properly.")
                        .color(ORANGE));

                });
        }
        });
        // private stats
        ui.add_space(space_h);
        ui.add_enabled_ui(private_stats, |ui| {
            ui.horizontal(|ui| {
                let priv_stats = &lock!(api).stats_priv;
                let width_stat = (ui.available_width() - SPACE * 4.0) / 5.0;
                let height_stat = 0.0;
                let size_stat = vec2(width_stat, height_stat);
                let round = match &priv_stats.round_participate {
                    Some(r) => r.to_string(),
                    None => "None".to_string(),
                };
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
                                ui.label(
                                    [
                                        Float::from_3(priv_stats.donor_1hr_avg as f64).to_string(),
                                        " kH/s".to_string(),
                                    ]
                                    .concat(),
                                );
                            })
                            .response
                        });
                        ui.separator();
                        ui.add_sized(size_stat, |ui: &mut Ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(XVB_DONATED_24H_FIELD);
                                ui.label(
                                    [
                                        Float::from_3(priv_stats.donor_24hr_avg as f64).to_string(),
                                        " kH/s".to_string(),
                                    ]
                                    .concat(),
                                );
                            })
                            .response
                        });
                        ui.separator();
                        ui.add_enabled_ui(priv_stats.round_participate.is_some(), |ui| {
                            ui.add_sized(size_stat, |ui: &mut Ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(XVB_ROUND_TYPE_FIELD);
                                    ui.label(round);
                                })
                                .response
                            })
                            .on_disabled_hover_text(
                                "You do not yet have a share in the PPLNS Window.",
                            );
                        });
                        ui.separator();
                        ui.add_sized(size_stat, |ui: &mut Ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(XVB_WINNER_FIELD);
                                ui.label(if priv_stats.win_current {
                                    "You are Winning the round !"
                                } else {
                                    "You are not the winner"
                                });
                            })
                            .response
                        });
                    })
                    .response
                });
            });
        });
        // Rules link help
        ui.horizontal_centered(|ui| {
            // can't have horizontal and vertical centering work together so fix by this.
            ui.add_space((width / 2.0) - (ui.text_style_height(&TextStyle::Heading) * 1.5));
            ui.style_mut().override_text_style = Some(TextStyle::Heading);
            ui.hyperlink_to("Rules", XVB_URL_RULES)
                .on_hover_text("Click here to read the rules and understand how the raffle works.");
        });
    }
}
