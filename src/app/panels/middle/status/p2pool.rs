use std::sync::{Arc, Mutex};

use egui::{Label, RichText, SelectableLabel, Slider, TextEdit, Vec2};
use readable::num::Unsigned;

use crate::{
    disk::{
        gupax_p2pool_api::GupaxP2poolApi,
        state::Status,
        status::{Hash, PayoutView},
    },
    helper::p2pool::PubP2poolApi,
    utils::constants::*,
};

impl Status {
    pub fn p2pool(
        &mut self,
        size: Vec2,
        ui: &mut egui::Ui,
        gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>,
        p2pool_alive: bool,
        p2pool_api: &Arc<Mutex<PubP2poolApi>>,
    ) {
        let api = gupax_p2pool_api.lock().unwrap();
        let height = size.y;
        let width = size.x;
        let text = height / 25.0;
        let log = height / 2.8;
        // Payout Text + PayoutView buttons
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let width = (width / 3.0) - (SPACE * 4.0);
                ui.add_sized(
                    [width, text],
                    Label::new(
                        RichText::new(format!("Total Payouts: {}", api.payout))
                            .underline()
                            .color(LIGHT_GRAY),
                    ),
                )
                .on_hover_text(STATUS_SUBMENU_PAYOUT);
                ui.separator();
                ui.add_sized(
                    [width, text],
                    Label::new(
                        RichText::new(format!("Total XMR: {}", api.xmr))
                            .underline()
                            .color(LIGHT_GRAY),
                    ),
                )
                .on_hover_text(STATUS_SUBMENU_XMR);
                let width = width / 4.0;
                ui.separator();
                if ui
                    .add_sized(
                        [width, text],
                        SelectableLabel::new(self.payout_view == PayoutView::Latest, "Latest"),
                    )
                    .on_hover_text(STATUS_SUBMENU_LATEST)
                    .clicked()
                {
                    self.payout_view = PayoutView::Latest;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, text],
                        SelectableLabel::new(self.payout_view == PayoutView::Oldest, "Oldest"),
                    )
                    .on_hover_text(STATUS_SUBMENU_OLDEST)
                    .clicked()
                {
                    self.payout_view = PayoutView::Oldest;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, text],
                        SelectableLabel::new(self.payout_view == PayoutView::Biggest, "Biggest"),
                    )
                    .on_hover_text(STATUS_SUBMENU_BIGGEST)
                    .clicked()
                {
                    self.payout_view = PayoutView::Biggest;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, text],
                        SelectableLabel::new(self.payout_view == PayoutView::Smallest, "Smallest"),
                    )
                    .on_hover_text(STATUS_SUBMENU_SMALLEST)
                    .clicked()
                {
                    self.payout_view = PayoutView::Smallest;
                }
            });
            ui.separator();
            // Actual logs
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(self.payout_view == PayoutView::Oldest)
                    .max_width(width)
                    .max_height(log)
                    .auto_shrink([false; 2])
                    .show_viewport(ui, |ui, _| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
                        match self.payout_view {
                            PayoutView::Latest => ui.add_sized(
                                [width, log],
                                TextEdit::multiline(&mut api.log_rev.as_str()),
                            ),
                            PayoutView::Oldest => ui.add_sized(
                                [width, log],
                                TextEdit::multiline(&mut api.log.as_str()),
                            ),
                            PayoutView::Biggest => ui.add_sized(
                                [width, log],
                                TextEdit::multiline(&mut api.payout_high.as_str()),
                            ),
                            PayoutView::Smallest => ui.add_sized(
                                [width, log],
                                TextEdit::multiline(&mut api.payout_low.as_str()),
                            ),
                        };
                    });
            });
        });
        drop(api);
        // Payout/Share Calculator
        let button = (width / 20.0) - (SPACE * 1.666);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(width - SPACE);
                if ui
                    .add_sized(
                        [button * 2.0, text],
                        SelectableLabel::new(!self.manual_hash, "Automatic"),
                    )
                    .on_hover_text(STATUS_SUBMENU_AUTOMATIC)
                    .clicked()
                {
                    self.manual_hash = false;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [button * 2.0, text],
                        SelectableLabel::new(self.manual_hash, "Manual"),
                    )
                    .on_hover_text(STATUS_SUBMENU_MANUAL)
                    .clicked()
                {
                    self.manual_hash = true;
                }
                ui.separator();
                ui.add_enabled_ui(self.manual_hash, |ui| {
                    if ui
                        .add_sized(
                            [button, text],
                            SelectableLabel::new(self.hash_metric == Hash::Hash, "Hash"),
                        )
                        .on_hover_text(STATUS_SUBMENU_HASH)
                        .clicked()
                    {
                        self.hash_metric = Hash::Hash;
                    }
                    ui.separator();
                    if ui
                        .add_sized(
                            [button, text],
                            SelectableLabel::new(self.hash_metric == Hash::Kilo, "Kilo"),
                        )
                        .on_hover_text(STATUS_SUBMENU_KILO)
                        .clicked()
                    {
                        self.hash_metric = Hash::Kilo;
                    }
                    ui.separator();
                    if ui
                        .add_sized(
                            [button, text],
                            SelectableLabel::new(self.hash_metric == Hash::Mega, "Mega"),
                        )
                        .on_hover_text(STATUS_SUBMENU_MEGA)
                        .clicked()
                    {
                        self.hash_metric = Hash::Mega;
                    }
                    ui.separator();
                    if ui
                        .add_sized(
                            [button, text],
                            SelectableLabel::new(self.hash_metric == Hash::Giga, "Giga"),
                        )
                        .on_hover_text(STATUS_SUBMENU_GIGA)
                        .clicked()
                    {
                        self.hash_metric = Hash::Giga;
                    }
                    ui.separator();
                    ui.spacing_mut().slider_width = button * 11.5;
                    ui.add_sized(
                        [button * 14.0, text],
                        Slider::new(&mut self.hashrate, 1.0..=1_000.0),
                    );
                });
            })
        });
        // Actual stats
        ui.add_enabled_ui(p2pool_alive, |ui| {
            let text = height / 25.0;
            let width = (width / 3.0) - (SPACE * 1.666);
            let min_height = ui.available_height() / 1.3;
            let api = p2pool_api.lock().unwrap();
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(min_height);
                        ui.add_sized(
                            [width, text],
                            Label::new(RichText::new("Monero Difficulty").underline().color(BONE)),
                        )
                        .on_hover_text(STATUS_SUBMENU_MONERO_DIFFICULTY);
                        ui.add_sized([width, text], Label::new(api.monero_difficulty.as_str()));
                        ui.add_sized(
                            [width, text],
                            Label::new(RichText::new("Monero Hashrate").underline().color(BONE)),
                        )
                        .on_hover_text(STATUS_SUBMENU_MONERO_HASHRATE);
                        ui.add_sized([width, text], Label::new(api.monero_hashrate.as_str()));
                        ui.add_sized(
                            [width, text],
                            Label::new(RichText::new("P2Pool Difficulty").underline().color(BONE)),
                        )
                        .on_hover_text(STATUS_SUBMENU_P2POOL_DIFFICULTY);
                        ui.add_sized([width, text], Label::new(api.p2pool_difficulty.as_str()));
                        ui.add_sized(
                            [width, text],
                            Label::new(RichText::new("P2Pool Hashrate").underline().color(BONE)),
                        )
                        .on_hover_text(STATUS_SUBMENU_P2POOL_HASHRATE);
                        ui.add_sized([width, text], Label::new(api.p2pool_hashrate.as_str()));
                    })
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(min_height);
                        if self.manual_hash {
                            let hashrate =
                                Hash::convert_to_hash(self.hashrate, self.hash_metric) as u64;
                            let p2pool_share_mean = PubP2poolApi::calculate_share_or_block_time(
                                hashrate,
                                api.p2pool_difficulty_u64,
                            );
                            let solo_block_mean = PubP2poolApi::calculate_share_or_block_time(
                                hashrate,
                                api.monero_difficulty_u64,
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Manually Inputted Hashrate")
                                        .underline()
                                        .color(BONE),
                                ),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(format!("{} H/s", Unsigned::from(hashrate))),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("P2Pool Block Mean").underline().color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_BLOCK_MEAN);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.p2pool_block_mean.to_string()),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your P2Pool Share Mean")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_SHARE_MEAN);
                            ui.add_sized([width, text], Label::new(p2pool_share_mean.to_string()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your Solo Block Mean")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_SOLO_BLOCK_MEAN);
                            ui.add_sized([width, text], Label::new(solo_block_mean.to_string()));
                        } else {
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your P2Pool Hashrate")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_HASHRATE);
                            ui.add_sized(
                                [width, text],
                                Label::new(format!("{} H/s", api.hashrate_1h)),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("P2Pool Block Mean").underline().color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_BLOCK_MEAN);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.p2pool_block_mean.to_string()),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your P2Pool Share Mean")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_SHARE_MEAN);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.p2pool_share_mean.to_string()),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your Solo Block Mean")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_SOLO_BLOCK_MEAN);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.solo_block_mean.to_string()),
                            );
                        }
                    })
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(min_height);
                        if self.manual_hash {
                            let hashrate =
                                Hash::convert_to_hash(self.hashrate, self.hash_metric) as u64;
                            let user_p2pool_percent = PubP2poolApi::calculate_dominance(
                                hashrate,
                                api.p2pool_hashrate_u64,
                            );
                            let user_monero_percent = PubP2poolApi::calculate_dominance(
                                hashrate,
                                api.monero_hashrate_u64,
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(RichText::new("P2Pool Miners").underline().color(BONE)),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_MINERS);
                            ui.add_sized([width, text], Label::new(api.miners.as_str()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("P2Pool Dominance").underline().color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_DOMINANCE);
                            ui.add_sized([width, text], Label::new(api.p2pool_percent.as_str()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your P2Pool Dominance")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_DOMINANCE);
                            ui.add_sized([width, text], Label::new(user_p2pool_percent.as_str()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your Monero Dominance")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_YOUR_MONERO_DOMINANCE);
                            ui.add_sized([width, text], Label::new(user_monero_percent.as_str()));
                        } else {
                            ui.add_sized(
                                [width, text],
                                Label::new(RichText::new("P2Pool Miners").underline().color(BONE)),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_MINERS);
                            ui.add_sized([width, text], Label::new(api.miners.as_str()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("P2Pool Dominance").underline().color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_P2POOL_DOMINANCE);
                            ui.add_sized([width, text], Label::new(api.p2pool_percent.as_str()));
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your P2Pool Dominance")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_DOMINANCE);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.user_p2pool_percent.as_str()),
                            );
                            ui.add_sized(
                                [width, text],
                                Label::new(
                                    RichText::new("Your Monero Dominance")
                                        .underline()
                                        .color(BONE),
                                ),
                            )
                            .on_hover_text(STATUS_SUBMENU_YOUR_MONERO_DOMINANCE);
                            ui.add_sized(
                                [width, text],
                                Label::new(api.user_monero_percent.as_str()),
                            );
                        }
                    })
                });
            });
            // Tick bar
            ui.add_sized(
                [ui.available_width(), text],
                Label::new(api.calculate_tick_bar()),
            )
            .on_hover_text(STATUS_SUBMENU_PROGRESS_BAR);
            drop(api);
        });
    }
}
