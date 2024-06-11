use std::sync::{Arc, Mutex};

use egui::TextStyle::{self, Name};
use egui::{vec2, Image, RichText, TextEdit, Ui, Vec2};
use log::debug;
use readable::num::Float;
use readable::up::Uptime;

use crate::disk::state::{XvbMode, ManualDonationLevel, ManualDonationMetric};
use crate::helper::xmrig::PubXmrigApi;
use crate::helper::xvb::priv_stats::RuntimeMode;
use crate::helper::xvb::PubXvbApi;
use crate::regex::num_lines;
use crate::utils::constants::{
    GREEN, LIGHT_GRAY, ORANGE, RED, XVB_DONATED_1H_FIELD, XVB_DONATED_24H_FIELD, XVB_FAILURE_FIELD,
    XVB_HELP, XVB_ROUND_TYPE_FIELD, XVB_TOKEN_FIELD, XVB_TOKEN_LEN, XVB_URL_RULES,
    XVB_WINNER_FIELD, XVB_HERO_SELECT, XVB_MODE_MANUALLY_DONATE, XVB_MODE_MANUALLY_KEEP, XVB_MODE_MANUAL_DONATION_LEVEL,
    XVB_DONATION_LEVEL_DONOR_HELP, XVB_DONATION_LEVEL_VIP_DONOR_HELP, XVB_DONATION_LEVEL_WHALE_DONOR_HELP,
    XVB_DONATION_LEVEL_MEGA_DONOR_HELP, XVB_MANUAL_SLIDER_DONATE_HELP, XVB_MANUAL_SLIDER_KEEP_HELP
};
use crate::utils::macros::lock;
use crate::utils::regex::Regexes;
use crate::{XVB_MINING_ON_FIELD};
use crate::{
    constants::{BYTES_XVB, SPACE},
    utils::constants::{DARK_GRAY, XVB_URL},
};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        size: Vec2,
        address: &str,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        api: &Arc<Mutex<PubXvbApi>>,
        gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
        is_alive: bool,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {

            let text_edit = size.y / 25.0;
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
            // hovering text is difficult because egui doesn't hover over inner widget. But on disabled does.
                    ui.group(|ui| {
                        ui.colored_label(color, text)
                        .on_hover_text(XVB_HELP);
                        ui.add(
                            TextEdit::singleline(&mut self.token)
                                .char_limit(9)
                                .desired_width(ui.text_style_height(&TextStyle::Body) * 9.0)
                                .vertical_align(egui::Align::Center),
                            ).on_hover_text(XVB_HELP)
                });
                // .on_hover_text(XVB_HELP);
                ui.add_space(height / 48.0);
        ui.style_mut().spacing.icon_width_inner = width / 45.0;
        ui.style_mut().spacing.icon_width = width / 35.0;
        ui.style_mut().spacing.icon_spacing = space_h;

        
        // --------------------------- XVB Simple -------------------------------------------
        if self.simple {

            if ui.checkbox(&mut self.simple_hero_mode, "Hero Mode").on_hover_text(XVB_HERO_SELECT).clicked() {
                // also change hero mode of runtime.
                lock!(api).stats_priv.runtime_mode = RuntimeMode::Hero;
            } else {
                lock!(api).stats_priv.runtime_mode = RuntimeMode::Auto;
            }

        }
    });


        ui.add_space(space_h);

         // --------------------------- XVB Advanced -----------------------------------------
         if !self.simple {

            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {

                        egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.mode))
                        .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.mode, XvbMode::Auto, "Automatic");
                                ui.selectable_value(&mut self.mode, XvbMode::Hero, "Hero Mode");
                                ui.selectable_value(&mut self.mode, XvbMode::ManuallyDonate, "Manually Donate")
                                .on_hover_text(XVB_MODE_MANUALLY_DONATE);
                                ui.selectable_value(&mut self.mode, XvbMode::ManuallyKeep, "Manually Keep")
                                .on_hover_text(XVB_MODE_MANUALLY_KEEP);
                                ui.selectable_value(&mut self.mode, XvbMode::ManualDonationLevel, "Manual Donation Level")
                                .on_hover_text(XVB_MODE_MANUAL_DONATION_LEVEL);
                        });
                        if self.mode == XvbMode::ManuallyDonate || self.mode == XvbMode::ManuallyKeep {

                            ui.add_space(space_h);

                            let mut hashrate_xmrig = {
                                if lock!(gui_api_xmrig).hashrate_raw_15m > 0.0 {
                                    lock!(gui_api_xmrig).hashrate_raw_15m
                                } else if lock!(gui_api_xmrig).hashrate_raw_1m > 0.0 {
                                    lock!(gui_api_xmrig).hashrate_raw_1m
                                } else {
                                    lock!(gui_api_xmrig).hashrate_raw
                                }
                            };

                            if self.manual_donation_metric == ManualDonationMetric::Kilo {
                                hashrate_xmrig /= 1000.0;
                            }
                            
                            let slider_help_text = if self.mode == XvbMode::ManuallyDonate {
                                XVB_MANUAL_SLIDER_DONATE_HELP
                            } else {
                                XVB_MANUAL_SLIDER_KEEP_HELP
                            };

                            ui.add_enabled_ui(is_alive, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().slider_width = width * 0.5;
                                    ui.add_sized(
                                        [width, text_edit],
                                        egui::Slider::new(&mut self.amount, 0.0..=(hashrate_xmrig as f64))
                                    ).on_hover_text(slider_help_text);

                                    if ui.add(egui::SelectableLabel::new(self.manual_donation_metric == ManualDonationMetric::Hash, "H/s")).clicked() {
                                        self.manual_donation_metric = ManualDonationMetric::Hash;
                                    }
                                    if ui.add(egui::SelectableLabel::new(self.manual_donation_metric == ManualDonationMetric::Kilo, "kH/s")).clicked() {
                                        self.manual_donation_metric = ManualDonationMetric::Kilo;
                                    };

                                });
                            });
                            
                        }
                        
                        if self.mode ==  XvbMode::ManualDonationLevel {
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::Donor, "Donor")
                            .on_hover_text(XVB_DONATION_LEVEL_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorVIP, "DonorVIP")
                            .on_hover_text(XVB_DONATION_LEVEL_VIP_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorWhale, "DonorWhale")
                            .on_hover_text(XVB_DONATION_LEVEL_WHALE_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorMega, "DonorMega")
                            .on_hover_text(XVB_DONATION_LEVEL_MEGA_DONOR_HELP);
                            
                            lock!(api).stats_priv.runtime_manual_donation_level = self.manual_donation_level.clone().into();
                        }

                    });
                });
            });

            // Set runtime_mode & runtime_manual_amount
            lock!(api).stats_priv.runtime_mode = self.mode.clone().into();
            if self.manual_donation_metric == ManualDonationMetric::Hash {
                lock!(api).stats_priv.runtime_manual_amount = self.amount;
            } else {
                lock!(api).stats_priv.runtime_manual_amount = self.amount * 1000.0;
            }
        } 

         ui.add_space(space_h);
        // need to warn the user if no address is set in p2pool tab
        if !Regexes::addr_ok(address) {
            debug!("XvB Tab | Rendering warning text");
                ui.horizontal_wrapped(|ui|{
            ui.label(RichText::new("You don't have any payout address set in the P2pool Tab ! XvB process needs one to function properly.")
                        .color(ORANGE));

                });
        }
            
            // private stats
            ui.add_space(space_h);
            // ui.add_enabled_ui(is_alive, |ui| {
            ui.add_enabled_ui(is_alive, |ui| {
                let api = &lock!(api);
                let priv_stats = &api.stats_priv;
                let current_node = &api.current_node;
                let width_stat = (ui.available_width() - SPACE * 4.0) / 5.0;
                let height_stat = 0.0;
                let size_stat = vec2(width_stat, height_stat);
                ui.horizontal(|ui| {
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
                // indicators
                ui.horizontal(|ui| {
                    ui.add_sized(size_stat, |ui: &mut Ui| {
                        ui.group(|ui| {
                            let size_stat = vec2(
                                ui.available_width(),
                                0.0, // + ui.spacing().item_spacing.y,
                            );
                            ui.add_sized(size_stat, |ui: &mut Ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(XVB_MINING_ON_FIELD)
                                        .on_hover_text_at_pointer(&priv_stats.msg_indicator);
                                    ui.label(
                                        current_node
                                            .as_ref()
                                            .map_or("No where".to_string(), |n| n.to_string()),
                                    )
                                    .on_hover_text_at_pointer(&priv_stats.msg_indicator);
                                    ui.label(Uptime::from(priv_stats.time_switch_node).to_string())
                                        .on_hover_text_at_pointer(&priv_stats.msg_indicator)
                                })
                                .response
                            })
                        })
                        .response
                        .on_disabled_hover_text("Algorithm is not running.")
                    })
                    // currently mining on
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

        });
    }
}
