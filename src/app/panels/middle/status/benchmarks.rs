use std::sync::{Arc, Mutex};

use crate::{app::Benchmark, disk::state::Status, helper::xmrig::PubXmrigApi};
use egui::{Hyperlink, ProgressBar, ScrollArea, Spinner, Vec2};
use egui_extras::{Column, TableBuilder};
use readable::num::{Float, Percent, Unsigned};

use crate::utils::macros::lock;

use crate::constants::*;
use egui::{Label, RichText};
use log::*;
impl Status {
    pub(super) fn benchmarks(
        &mut self,
        size: Vec2,
        ui: &mut egui::Ui,
        benchmarks: &[Benchmark],
        xmrig_alive: bool,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
    ) {
        debug!("Status Tab | Rendering [Benchmarks]");
        let text = size.y / 20.0;
        let double = text * 2.0;
        let log = size.y / 3.0;

        let width = size.x;
        // [0], The user's CPU (most likely).
        let cpu = &benchmarks[0];
        ui.horizontal(|ui| {
            let width = (width / 2.0) - (SPACE * 1.666);
            let min_height = log;
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_height(min_height);
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Your CPU").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_CPU);
                    ui.add_sized([width, text], Label::new(cpu.cpu.as_str()));
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Total Benchmarks").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_BENCHMARKS);
                    ui.add_sized([width, text], Label::new(format!("{}", cpu.benchmarks)));
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Rank").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_RANK);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{}/{}", cpu.rank, &benchmarks.len())),
                    );
                })
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_height(min_height);
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("High Hashrate").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_HIGH);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{} H/s", Float::from_0(cpu.high.into()))),
                    );
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Average Hashrate").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_AVERAGE);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{} H/s", Float::from_0(cpu.average.into()))),
                    );
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Low Hashrate").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_LOW);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{} H/s", Float::from_0(cpu.low.into()))),
                    );
                })
            })
        });

        // User's CPU hashrate comparison (if XMRig is alive).
        ui.scope(|ui| {
            if xmrig_alive {
                let api = lock!(xmrig_api);
                let percent = (api.hashrate_raw / cpu.high) * 100.0;
                let human = Percent::from(percent);
                if percent > 100.0 {
                    ui.add_sized(
                        [width, double],
                        Label::new(format!(
                        "Your CPU's is faster than the highest benchmark! It is [{}] faster @ {}!",
                        human, api.hashrate
                    )),
                    );
                    ui.add_sized([width, text], ProgressBar::new(1.0));
                } else if api.hashrate_raw == 0.0 {
                    ui.add_sized([width, text], Label::new("Measuring hashrate..."));
                    ui.add_sized([width, text], Spinner::new().size(text));
                    ui.add_sized([width, text], ProgressBar::new(0.0));
                } else {
                    ui.add_sized(
                        [width, double],
                        Label::new(format!(
                            "Your CPU's hashrate is [{}] of the highest benchmark @ {}",
                            human, api.hashrate
                        )),
                    );
                    ui.add_sized([width, text], ProgressBar::new(percent / 100.0));
                }
            } else {
                ui.set_enabled(xmrig_alive);
                ui.add_sized(
                    [width, double],
                    Label::new("XMRig is offline. Hashrate cannot be determined."),
                );
                ui.add_sized([width, text], ProgressBar::new(0.0));
            }
        });

        // Comparison
        ui.group(|ui| {
            ui.add_sized(
                [width, text],
                Hyperlink::from_label_and_url("Other CPUs", "https://xmrig.com/benchmark"),
            )
            .on_hover_text(STATUS_SUBMENU_OTHER_CPUS);
        });
        let width_column = width / 20.0;
        let (cpu, bar, high, average, low, rank, bench) = (
            width_column * 10.0,
            width_column * 3.0,
            width_column * 2.0,
            width_column * 2.0,
            width_column * 2.0,
            width_column,
            width_column * 2.0,
        );
        ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                .columns(Column::auto(), 7)
                .header(double, |mut header| {
                    header.col(|ui| {
                        ui.add_sized([bar, text], Label::new("CPU"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_CPU);
                    });
                    header.col(|ui| {
                        ui.add_sized([bar, text], Label::new("Relative"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_RELATIVE);
                    });
                    header.col(|ui| {
                        ui.add_sized([high, text], Label::new("High"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_HIGH);
                    });
                    header.col(|ui| {
                        ui.add_sized([average, text], Label::new("Average"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_AVERAGE);
                    });
                    header.col(|ui| {
                        ui.add_sized([low, text], Label::new("Low"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_LOW);
                    });
                    header.col(|ui| {
                        ui.add_sized([rank, text], Label::new("Rank"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_RANK);
                    });
                    header.col(|ui| {
                        ui.add_sized([bench, text], Label::new("Benchmarks"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_BENCHMARKS);
                    });
                })
                .body(|body| {
                    body.rows(text, benchmarks.len(), |mut row| {
                        let benchmark = &benchmarks[row.index()];
                        row.col(|ui| {
                            ui.add_sized([cpu, text], Label::new(benchmark.cpu.as_str()));
                        });
                        row.col(|ui| {
                            ui.add_sized([bar, text], ProgressBar::new(benchmark.percent / 100.0))
                                .on_hover_text(Percent::from(benchmark.percent).as_str());
                        });
                        row.col(|ui| {
                            ui.add_sized(
                                [high, text],
                                Label::new(
                                    [Float::from_0(benchmark.high.into()).as_str(), " H/s"]
                                        .concat(),
                                ),
                            );
                        });
                        row.col(|ui| {
                            ui.add_sized(
                                [average, text],
                                Label::new(
                                    [Float::from_0(benchmark.average.into()).as_str(), " H/s"]
                                        .concat(),
                                ),
                            );
                        });
                        row.col(|ui| {
                            ui.add_sized(
                                [low, text],
                                Label::new(
                                    [Float::from_0(benchmark.low.into()).as_str(), " H/s"].concat(),
                                ),
                            );
                        });
                        row.col(|ui| {
                            ui.add_sized(
                                [rank, text],
                                Label::new(Unsigned::from(benchmark.rank).as_str()),
                            );
                        });
                        row.col(|ui| {
                            ui.add_sized(
                                [bench, text],
                                Label::new(Unsigned::from(benchmark.benchmarks).as_str()),
                            );
                        });
                    });
                });
        });
    }
}
