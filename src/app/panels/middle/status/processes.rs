use egui::{ScrollArea, Ui, Vec2};
use readable::up::UptimeFull;
use std::sync::{Arc, Mutex};

use crate::disk::state::Status;
use crate::helper::p2pool::{ImgP2pool, PubP2poolApi};
use crate::helper::xmrig::{ImgXmrig, PubXmrigApi};
use crate::helper::xvb::{PubXvbApi, XvbRound};
use crate::helper::Sys;
use crate::utils::macros::lock;
use egui::TextStyle;

use crate::constants::*;
use egui::{Label, RichText, TextStyle::*};
use log::*;
impl Status {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn processes(
        &mut self,
        sys: &Arc<Mutex<Sys>>,
        size: Vec2,
        ui: &mut egui::Ui,
        p2pool_alive: bool,
        p2pool_api: &Arc<Mutex<PubP2poolApi>>,
        p2pool_img: &Arc<Mutex<ImgP2pool>>,
        xmrig_alive: bool,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
        xmrig_img: &Arc<Mutex<ImgXmrig>>,
        xvb_alive: bool,
        xvb_api: &Arc<Mutex<PubXvbApi>>,
        max_threads: usize,
    ) {
        let width = (size.x / 4.0) - (SPACE * 1.7500);
        let min_height = size.y - SPACE;
        let height = size.y / 25.0;
        ui.horizontal(|ui| {
            // [Gupax]
            gupax(ui, min_height, width, height, sys);
            // [P2Pool]
            p2pool(
                ui,
                min_height,
                width,
                height,
                p2pool_alive,
                p2pool_api,
                p2pool_img,
            );
            // [XMRig]
            xmrig(
                ui,
                min_height,
                width,
                height,
                xmrig_alive,
                xmrig_api,
                xmrig_img,
                max_threads,
            );
            // [XvB]
            xvb(ui, min_height, width, height, xvb_alive, xvb_api);
        });
    }
}
fn gupax(ui: &mut Ui, min_height: f32, width: f32, height: f32, sys: &Arc<Mutex<Sys>>) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            debug!("Status Tab | Rendering [Gupax]");
            ui.set_min_height(min_height);
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("[Gupax]")
                        .color(LIGHT_GRAY)
                        .text_style(TextStyle::Name("MonospaceLarge".into())),
                ),
            )
            .on_hover_text("Gupax is online");
            let sys = lock!(sys);
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Uptime").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_UPTIME);
            ui.add_sized([width, height], Label::new(sys.gupax_uptime.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Gupax CPU").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_CPU_USAGE);
            ui.add_sized([width, height], Label::new(sys.gupax_cpu_usage.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Gupax Memory").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
            ui.add_sized(
                [width, height],
                Label::new(sys.gupax_memory_used_mb.to_string()),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("System CPU").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
            ui.add_sized(
                [width, height],
                Label::new(sys.system_cpu_usage.to_string()),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("System Memory").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
            ui.add_sized([width, height], Label::new(sys.system_memory.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("System CPU Model").underline().color(BONE)),
            )
            .on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
            ui.add_sized(
                [width, height],
                Label::new(sys.system_cpu_model.to_string()),
            );
            drop(sys);
        })
    });
}

fn p2pool(
    ui: &mut Ui,
    min_height: f32,
    width: f32,
    height: f32,
    p2pool_alive: bool,
    p2pool_api: &Arc<Mutex<PubP2poolApi>>,
    p2pool_img: &Arc<Mutex<ImgP2pool>>,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            debug!("Status Tab | Rendering [P2Pool]");
            ui.set_enabled(p2pool_alive);
            ui.set_min_height(min_height);
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("[P2Pool]")
                        .color(LIGHT_GRAY)
                        .text_style(TextStyle::Name("MonospaceLarge".into())),
                ),
            )
            .on_hover_text("P2Pool is online")
            .on_disabled_hover_text("P2Pool is offline");
            ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
            let height = height / 1.4;
            let api = lock!(p2pool_api);
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Uptime").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_UPTIME);
            ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Shares Found").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_SHARES);
            ui.add_sized(
                [width, height],
                Label::new(
                    (if let Some(s) = api.shares_found {
                        s.to_string()
                    } else {
                        UNKNOWN_DATA.to_string()
                    })
                    .to_string(),
                ),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Payouts").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_PAYOUTS);
            ui.add_sized(
                [width, height],
                Label::new(format!("Total: {}", api.payouts)),
            );
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]",
                    api.payouts_hour, api.payouts_day, api.payouts_month
                )),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("XMR Mined").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_XMR);
            ui.add_sized(
                [width, height],
                Label::new(format!("Total: {:.13} XMR", api.xmr)),
            );
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]",
                    api.xmr_hour, api.xmr_day, api.xmr_month
                )),
            );
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("Hashrate (15m/1h/24h)")
                        .underline()
                        .color(BONE),
                ),
            )
            .on_hover_text(STATUS_P2POOL_HASHRATE);
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[{} H/s] [{} H/s] [{} H/s]",
                    api.hashrate_15m, api.hashrate_1h, api.hashrate_24h
                )),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Miners Connected").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_CONNECTIONS);
            ui.add_sized([width, height], Label::new(format!("{}", api.connections)));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Effort").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_EFFORT);
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[Average: {}] [Current: {}]",
                    api.average_effort, api.current_effort
                )),
            );
            let img = lock!(p2pool_img);
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Monero Node").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_MONERO_NODE);
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[IP: {}]\n[RPC: {}] [ZMQ: {}]",
                    &img.host, &img.rpc, &img.zmq
                )),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Sidechain").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_POOL);
            ui.add_sized([width, height], Label::new(&img.mini));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Address").underline().color(BONE)),
            )
            .on_hover_text(STATUS_P2POOL_ADDRESS);
            ui.add_sized([width, height], Label::new(&img.address));
            drop(img);
            drop(api);
        })
    });
}
#[allow(clippy::too_many_arguments)]
fn xmrig(
    ui: &mut Ui,
    min_height: f32,
    width: f32,
    height: f32,
    xmrig_alive: bool,
    xmrig_api: &Arc<Mutex<PubXmrigApi>>,
    xmrig_img: &Arc<Mutex<ImgXmrig>>,
    max_threads: usize,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            debug!("Status Tab | Rendering [XMRig]");
            ui.set_enabled(xmrig_alive);
            ui.set_min_height(min_height);
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("[XMRig]")
                        .color(LIGHT_GRAY)
                        .text_style(TextStyle::Name("MonospaceLarge".into())),
                ),
            )
            .on_hover_text("XMRig is online")
            .on_disabled_hover_text("XMRig is offline");
            let api = lock!(xmrig_api);
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Uptime").underline().color(BONE)),
            )
            .on_hover_text(STATUS_XMRIG_UPTIME);
            ui.add_sized(
                [width, height],
                Label::new(UptimeFull::from(api.uptime).as_str()),
            );
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("CPU Load (10s/60s/15m)")
                        .underline()
                        .color(BONE),
                ),
            )
            .on_hover_text(STATUS_XMRIG_CPU);
            ui.add_sized([width, height], Label::new(api.resources.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(
                    RichText::new("Hashrate (10s/60s/15m)")
                        .underline()
                        .color(BONE),
                ),
            )
            .on_hover_text(STATUS_XMRIG_HASHRATE);
            ui.add_sized([width, height], Label::new(api.hashrate.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Difficulty").underline().color(BONE)),
            )
            .on_hover_text(STATUS_XMRIG_DIFFICULTY);
            ui.add_sized([width, height], Label::new(api.diff.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Shares").underline().color(BONE)),
            )
            .on_hover_text(STATUS_XMRIG_SHARES);
            ui.add_sized(
                [width, height],
                Label::new(format!(
                    "[Accepted: {}] [Rejected: {}]",
                    api.accepted, api.rejected
                )),
            );
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Pool").underline().color(BONE)),
            )
            .on_hover_text(STATUS_XMRIG_POOL);
            ui.add_sized([width, height], Label::new(api.node.to_string()));
            ui.add_sized(
                [width, height],
                Label::new(RichText::new("Threads").underline().color(BONE)),
            )
            .on_hover_text(STATUS_XMRIG_THREADS);
            ui.add_sized(
                [width, height],
                Label::new(format!("{}/{}", &lock!(xmrig_img).threads, max_threads)),
            );
            drop(api);
        })
    });
}

fn xvb(
    ui: &mut Ui,
    min_height: f32,
    width: f32,
    height: f32,
    xvb_alive: bool,
    xvb_api: &Arc<Mutex<PubXvbApi>>,
) {
    //
    let api = &lock!(xvb_api).stats_pub;
    let enabled = xvb_alive;
    ScrollArea::vertical().show(ui, |ui| {
        ui.group(|ui| {
            ui.vertical(|ui| {
                debug!("Status Tab | Rendering [XvB]");
                ui.set_enabled(enabled); // for now there is no API ping or /health, so we verify if the field reward_yearly is empty or not.
                ui.set_min_height(min_height);
                ui.add_sized(
                    [width, height],
                    Label::new(
                        RichText::new("[XvB Raffle]")
                            .color(LIGHT_GRAY)
                            .text_style(TextStyle::Name("MonospaceLarge".into())),
                    ),
                )
                .on_hover_text("XvB API stats")
                .on_disabled_hover_text("No data received from XvB API");
                // [Round Type]
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Round Type").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_ROUND_TYPE);
                ui.add_sized([width, height], Label::new(api.round_type.to_string()));
                // [Time Remaining]
                ui.add_sized(
                    [width, height],
                    Label::new(
                        RichText::new("Round Time Remaining")
                            .underline()
                            .color(BONE),
                    ),
                )
                .on_hover_text(STATUS_XVB_TIME_REMAIN);
                ui.add_sized(
                    [width, height],
                    Label::new(format!("{} minutes", api.time_remain)),
                );
                // Donated Hashrate
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Bonus Hashrate").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_DONATED_HR);
                ui.add_sized(
                    [width, height],
                    Label::new(format!(
                        "{}kH/s\n+\n{}kH/s\ndonated by\n{} donors\n with\n{} miners",
                        api.bonus_hr, api.donate_hr, api.donate_miners, api.donate_workers
                    )),
                );
                // Players
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Players").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_PLAYERS);
                ui.add_sized(
                    [width, height],
                    Label::new(format!(
                        "[Registered: {}]\n[Playing: {}]",
                        api.players, api.players_round
                    )),
                );
                // Winner
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Winner").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_WINNER);
                ui.add_sized([width, height], Label::new(&api.winner));
                // Share effort
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Share Effort").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_SHARE);
                ui.add_sized([width, height], Label::new(api.share_effort.to_string()));
                // Block reward
                ui.add_sized(
                    [width, height],
                    Label::new(RichText::new("Block Reward").underline().color(BONE)),
                )
                .on_hover_text(STATUS_XVB_BLOCK_REWARD);
                ui.add_sized([width, height], Label::new(api.block_reward.to_string()));
                // reward yearly
                ui.add_sized(
                    [width, height],
                    Label::new(
                        RichText::new("Est. Reward (Yearly)")
                            .underline()
                            .color(BONE),
                    ),
                )
                .on_hover_text(STATUS_XVB_YEARLY);
                if api.reward_yearly.is_empty() {
                    ui.add_sized([width, height], Label::new("No information".to_string()));
                } else {
                    ui.add_sized(
                        [width, height],
                        Label::new(format!(
                            "{}: {} XMR\n{}: {} XMR\n{}: {} XMR\n{}: {} XMR\n{}: {} XMR",
                            XvbRound::Vip,
                            api.reward_yearly[0],
                            XvbRound::Donor,
                            api.reward_yearly[1],
                            XvbRound::DonorVip,
                            api.reward_yearly[2],
                            XvbRound::DonorWhale,
                            api.reward_yearly[3],
                            XvbRound::DonorMega,
                            api.reward_yearly[4]
                        )),
                    );
                }
            });
            // by round
        });
    });
}
