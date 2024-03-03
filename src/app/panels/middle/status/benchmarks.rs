use std::sync::{Arc, Mutex};

use crate::{
    app::Benchmark, disk::state::Status, helper::xmrig::PubXmrigApi, utils::human::HumanNumber,
};
use egui::{Hyperlink, ProgressBar, Spinner};

use crate::utils::macros::lock;

use crate::constants::*;
use egui::{Label, RichText};
use log::*;
impl Status {
    pub(super) fn benchmarks(
        &mut self,
        width: f32,
        height: f32,
        ui: &mut egui::Ui,
        benchmarks: &[Benchmark],
        xmrig_alive: bool,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
    ) {
        debug!("Status Tab | Rendering [Benchmarks]");
        let text = height / 20.0;
        let double = text * 2.0;
        let log = height / 3.0;

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
                        Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.high))),
                    );
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Average Hashrate").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_AVERAGE);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.average))),
                    );
                    ui.add_sized(
                        [width, text],
                        Label::new(RichText::new("Low Hashrate").underline().color(BONE)),
                    )
                    .on_hover_text(STATUS_SUBMENU_YOUR_LOW);
                    ui.add_sized(
                        [width, text],
                        Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.low))),
                    );
                })
            })
        });

        // User's CPU hashrate comparison (if XMRig is alive).
        ui.scope(|ui| {
            if xmrig_alive {
                let api = lock!(xmrig_api);
                let percent = (api.hashrate_raw / cpu.high) * 100.0;
                let human = HumanNumber::to_percent(percent);
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

        egui::ScrollArea::both()
            .scroll_bar_visibility(
                egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
            )
            .max_width(width)
            .max_height(height)
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, _| {
                let width = width / 20.0;
                let (cpu, bar, high, average, low, rank, bench) = (
                    width * 10.0,
                    width * 3.0,
                    width * 2.0,
                    width * 2.0,
                    width * 2.0,
                    width,
                    width * 2.0,
                );
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized([cpu, double], Label::new("CPU"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_CPU);
                        ui.separator();
                        ui.add_sized([bar, double], Label::new("Relative"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_RELATIVE);
                        ui.separator();
                        ui.add_sized([high, double], Label::new("High"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_HIGH);
                        ui.separator();
                        ui.add_sized([average, double], Label::new("Average"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_AVERAGE);
                        ui.separator();
                        ui.add_sized([low, double], Label::new("Low"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_LOW);
                        ui.separator();
                        ui.add_sized([rank, double], Label::new("Rank"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_RANK);
                        ui.separator();
                        ui.add_sized([bench, double], Label::new("Benchmarks"))
                            .on_hover_text(STATUS_SUBMENU_OTHER_BENCHMARKS);
                    });
                });

                for benchmark in benchmarks[1..].iter() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized([cpu, text], Label::new(benchmark.cpu.as_str()));
                            ui.separator();
                            ui.add_sized([bar, text], ProgressBar::new(benchmark.percent / 100.0))
                                .on_hover_text(HumanNumber::to_percent(benchmark.percent).as_str());
                            ui.separator();
                            ui.add_sized(
                                [high, text],
                                Label::new(HumanNumber::to_hashrate(benchmark.high).as_str()),
                            );
                            ui.separator();
                            ui.add_sized(
                                [average, text],
                                Label::new(HumanNumber::to_hashrate(benchmark.average).as_str()),
                            );
                            ui.separator();
                            ui.add_sized(
                                [low, text],
                                Label::new(HumanNumber::to_hashrate(benchmark.low).as_str()),
                            );
                            ui.separator();
                            ui.add_sized(
                                [rank, text],
                                Label::new(HumanNumber::from_u16(benchmark.rank).as_str()),
                            );
                            ui.separator();
                            ui.add_sized(
                                [bench, text],
                                Label::new(HumanNumber::from_u16(benchmark.benchmarks).as_str()),
                            );
                        })
                    });
                }
            });
    }
}
