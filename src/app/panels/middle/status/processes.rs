use egui::{ScrollArea, Ui};
use readable::up::UptimeFull;
use std::sync::{Arc, Mutex};

use crate::app::eframe_impl::ProcessStatesGui;
use crate::disk::state::Status;
use crate::helper::node::PubNodeApi;
use crate::helper::p2pool::{ImgP2pool, PubP2poolApi};
use crate::helper::xrig::xmrig::{ImgXmrig, PubXmrigApi};
use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::helper::xvb::{PubXvbApi, rounds::XvbRound};
use crate::helper::{ProcessName, Sys};

use crate::constants::*;
use egui::{RichText, TextStyle};
use log::*;
impl Status {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn processes(
        &mut self,
        sys: &Arc<Mutex<Sys>>,
        ui: &mut egui::Ui,
        node_api: &Arc<Mutex<PubNodeApi>>,
        p2pool_api: &Arc<Mutex<PubP2poolApi>>,
        p2pool_img: &Arc<Mutex<ImgP2pool>>,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
        xmrig_proxy_api: &Arc<Mutex<PubXmrigProxyApi>>,
        xmrig_img: &Arc<Mutex<ImgXmrig>>,
        xvb_api: &Arc<Mutex<PubXvbApi>>,
        max_threads: u16,
        states: &ProcessStatesGui,
    ) {
        let width_column = ui.text_style_height(&TextStyle::Body) * 16.0;
        let height_column = width_column * 2.5;
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        // let width = ((ui.available_width() / 5.0) - (SPACE * 1.7500)).max(0.0);
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                // ui.set_min_width(ui.text_style_height(&TextStyle::Body) * 2.0);
                                gupax(ui, sys);
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                node(ui, states.is_alive(ProcessName::Node), node_api);
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                p2pool(
                                    ui,
                                    states.is_alive(ProcessName::P2pool),
                                    p2pool_api,
                                    p2pool_img,
                                );
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                xmrig(
                                    ui,
                                    states.is_alive(ProcessName::Xmrig),
                                    xmrig_api,
                                    xmrig_img,
                                    max_threads,
                                );
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                xmrig_proxy(
                                    ui,
                                    states.is_alive(ProcessName::XmrigProxy),
                                    xmrig_proxy_api,
                                );
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                xvb(ui, states.is_alive(ProcessName::Xvb), xvb_api);
                            });
                        });
                    });
                });
            });
        });
    }
}

fn gupax(ui: &mut Ui, sys: &Arc<Mutex<Sys>>) {
    ui.label(
        RichText::new("[Gupaxx]")
            .color(LIGHT_GRAY)
            .text_style(TextStyle::Heading),
    )
    .on_hover_text("Gupaxx is online");
    let sys = sys.lock().unwrap();
    ui.label(RichText::new("Uptime").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_UPTIME);
    ui.label(sys.gupax_uptime.to_string());
    ui.label(RichText::new("Gupaxx CPU").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_CPU_USAGE);
    ui.label(sys.gupax_cpu_usage.to_string());
    ui.label(RichText::new("Gupaxx Memory").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
    ui.label(sys.gupax_memory_used_mb.to_string());
    ui.label(RichText::new("System CPU").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
    ui.label(sys.system_cpu_usage.to_string());
    ui.label(RichText::new("System Memory").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
    ui.label(sys.system_memory.to_string());
    ui.label(RichText::new("System CPU Model").underline().color(BONE))
        .on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    ui.label(sys.system_cpu_model.to_string());
    drop(sys);
}

fn p2pool(
    ui: &mut Ui,
    p2pool_alive: bool,
    p2pool_api: &Arc<Mutex<PubP2poolApi>>,
    p2pool_img: &Arc<Mutex<ImgP2pool>>,
) {
    ui.add_enabled_ui(p2pool_alive, |ui| {
        ui.label(
            RichText::new("[P2Pool]")
                .color(LIGHT_GRAY)
                .text_style(TextStyle::Heading),
        )
        .on_hover_text("P2Pool is online")
        .on_disabled_hover_text("P2Pool is offline");
        ui.style_mut().override_text_style = Some(TextStyle::Small);
        let api = p2pool_api.lock().unwrap();
        ui.label(RichText::new("Uptime").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_UPTIME);
        ui.label(format!("{}", api.uptime));
        ui.label(RichText::new("Current Shares").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_CURRENT_SHARES);
        ui.label(api.sidechain_shares.to_string());
        ui.label(RichText::new("Shares Found").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_SHARES);
        ui.label(
            (if let Some(s) = api.shares_found {
                s.to_string()
            } else {
                UNKNOWN_DATA.to_string()
            })
            .to_string(),
        );
        ui.label(RichText::new("Payouts").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_PAYOUTS);
        ui.label(format!("Total: {}", api.payouts));
        ui.label(format!(
            "[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]",
            api.payouts_hour, api.payouts_day, api.payouts_month
        ));
        ui.label(RichText::new("XMR Mined").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_XMR);
        ui.label(format!("Total: {:.13} XMR", api.xmr));
        ui.label(format!(
            "[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]",
            api.xmr_hour, api.xmr_day, api.xmr_month
        ));
        ui.label(
            RichText::new("Hashrate (15m/1h/24h)")
                .underline()
                .color(BONE),
        )
        .on_hover_text(STATUS_P2POOL_HASHRATE);
        ui.label(format!(
            "[{} H/s]\n[{} H/s]\n[{} H/s]",
            api.hashrate_15m, api.hashrate_1h, api.hashrate_24h
        ));
        ui.label(RichText::new("Miners Connected").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_CONNECTIONS);
        ui.label(format!("{}", api.connections));
        ui.label(RichText::new("Effort").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_EFFORT);
        ui.label(format!(
            "[Average: {}] [Current: {}]",
            api.average_effort, api.current_effort
        ));
        let img = p2pool_img.lock().unwrap();
        ui.label(RichText::new("Monero Node").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_MONERO_NODE);
        ui.label(format!(
            "IP: {}]\n[RPC: {}] [ZMQ: {}]",
            &img.host, &img.rpc, &img.zmq
        ));
        ui.label(RichText::new("Sidechain").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_POOL);
        ui.label(&img.mini);
        ui.label(RichText::new("Address").underline().color(BONE))
            .on_hover_text(STATUS_P2POOL_ADDRESS);
        ui.label(&img.address);
        drop(img);
        drop(api);
    });
}
#[allow(clippy::too_many_arguments)]
fn xmrig_proxy(
    ui: &mut Ui,
    xmrig_proxy_alive: bool,
    xmrig_proxy_api: &Arc<Mutex<PubXmrigProxyApi>>,
) {
    ui.add_enabled_ui(xmrig_proxy_alive, |ui| {
        ui.label(
            RichText::new("[XMRig-Proxy]")
                .color(LIGHT_GRAY)
                .text_style(TextStyle::Heading),
        )
        .on_hover_text("XMRig-Proxy is online")
        .on_disabled_hover_text("XMRig-Proxy is offline");
        let api = xmrig_proxy_api.lock().unwrap();
        ui.label(RichText::new("Uptime").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_PROXY_UPTIME);
        ui.label(UptimeFull::from(api.uptime).as_str());
        ui.label(
            RichText::new("Hashrate\n(1m/10m/1h/12h/24h)")
                .underline()
                .color(BONE),
        )
        .on_hover_text(STATUS_XMRIG_PROXY_HASHRATE);
        ui.label(format!(
            "[{} H/s]\n[{} H/s]\n[{} H/s]\n[{} H/s]\n[{} H/s]",
            api.hashrate_1m, api.hashrate_10m, api.hashrate_1h, api.hashrate_12h, api.hashrate_24h
        ));
        ui.label(format!(
            "[Accepted: {}]\n[Rejected: {}]",
            api.accepted, api.rejected
        ));
        ui.label(RichText::new("Pool").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_PROXY_POOL);
        ui.label(api.node.to_string());
        drop(api);
    });
}
#[allow(clippy::too_many_arguments)]
fn xmrig(
    ui: &mut Ui,
    xmrig_alive: bool,
    xmrig_api: &Arc<Mutex<PubXmrigApi>>,
    xmrig_img: &Arc<Mutex<ImgXmrig>>,
    max_threads: u16,
) {
    debug!("Status Tab | Rendering [XMRig]");
    ui.add_enabled_ui(xmrig_alive, |ui| {
        // ui.set_min_size(min_size);
        ui.label(
            RichText::new("[XMRig]")
                .color(LIGHT_GRAY)
                .text_style(TextStyle::Heading),
        )
        .on_hover_text("XMRig is online")
        .on_disabled_hover_text("XMRig is offline");
        let api = xmrig_api.lock().unwrap();
        ui.label(RichText::new("Uptime").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_UPTIME);
        ui.label(UptimeFull::from(api.uptime).as_str());
        ui.label(api.resources.to_string());
        ui.label(
            RichText::new("Hashrate\n(10s/1m/15m)")
                .underline()
                .color(BONE),
        )
        .on_hover_text(STATUS_XMRIG_HASHRATE);
        ui.label(api.hashrate.to_string());
        ui.label(RichText::new("Difficulty").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_DIFFICULTY);
        ui.label(api.diff.to_string());
        ui.label(RichText::new("Shares").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_SHARES);
        ui.label(format!(
            "[Accepted: {}]\n[Rejected: {}]",
            api.accepted, api.rejected
        ));
        ui.label(RichText::new("Pool").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_POOL);
        ui.label(api.node.to_string());
        ui.label(RichText::new("Threads").underline().color(BONE))
            .on_hover_text(STATUS_XMRIG_THREADS);
        ui.label(format!(
            "{}/{}",
            &xmrig_img.lock().unwrap().threads,
            max_threads
        ));
        drop(api);
    });
}

fn xvb(ui: &mut Ui, xvb_alive: bool, xvb_api: &Arc<Mutex<PubXvbApi>>) {
    //
    let api = &xvb_api.lock().unwrap().stats_pub;
    let enabled = xvb_alive;
    debug!("Status Tab | Rendering [XvB]");
    ui.add_enabled_ui(enabled, |ui| {
        // for now there is no API ping or /health, so we verify if the field reward_yearly is empty or not.
        // ui.set_min_size(min_size);
        ui.label(
            RichText::new("[XvB Raffle]")
                .color(LIGHT_GRAY)
                .text_style(TextStyle::Heading),
        )
        .on_hover_text("XvB API stats")
        .on_disabled_hover_text("No data received from XvB API");
        // [Round Type]
        ui.label(RichText::new("Round Type").underline().color(BONE))
            .on_hover_text(STATUS_XVB_ROUND_TYPE);
        ui.label(api.round_type.to_string());
        // [Time Remaining]
        ui.label(
            RichText::new("Round Time Remaining")
                .underline()
                .color(BONE),
        )
        .on_hover_text(STATUS_XVB_TIME_REMAIN);
        ui.label(format!("{} minutes", api.time_remain));
        // Donated Hashrate
        ui.label(RichText::new("Bonus Hashrate").underline().color(BONE))
            .on_hover_text(STATUS_XVB_DONATED_HR);
        ui.label(format!(
            "{}kH/s\n+\n{}kH/s\ndonated by\n{} donors\n with\n{} miners",
            api.bonus_hr, api.donate_hr, api.donate_miners, api.donate_workers
        ));
        // Players
        ui.label(RichText::new("Players").underline().color(BONE))
            .on_hover_text(STATUS_XVB_PLAYERS);
        ui.label(format!(
            "[Registered: {}]\n[Playing: {}]",
            api.players, api.players_round
        ));
        // Winner
        ui.label(RichText::new("Winner").underline().color(BONE))
            .on_hover_text(STATUS_XVB_WINNER);
        ui.label(&api.winner);
        // Share effort
        ui.label(RichText::new("Share Effort").underline().color(BONE))
            .on_hover_text(STATUS_XVB_SHARE);
        ui.label(api.share_effort.to_string());
        // Block reward
        ui.label(RichText::new("Block Reward").underline().color(BONE))
            .on_hover_text(STATUS_XVB_BLOCK_REWARD);
        ui.label(api.block_reward.to_string());
        // reward yearly
        ui.label(
            RichText::new("Est. Reward (Yearly)")
                .underline()
                .color(BONE),
        )
        .on_hover_text(STATUS_XVB_YEARLY);
        if api.reward_yearly.is_empty() {
            ui.label("No information".to_string());
        } else {
            ui.label(format!(
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
            ));
        }
    });
}
#[allow(clippy::too_many_arguments)]
fn node(ui: &mut Ui, node_alive: bool, node_api: &Arc<Mutex<PubNodeApi>>) {
    debug!("Status Tab | Rendering [Node]");
    ui.add_enabled_ui(node_alive, |ui| {
        ui.label(
            RichText::new("[Node]")
                .color(LIGHT_GRAY)
                .text_style(TextStyle::Heading),
        )
        .on_hover_text("Node is online")
        .on_disabled_hover_text("Node is offline");
        let api = node_api.lock().unwrap();
        ui.label(RichText::new("Uptime").underline().color(BONE))
            .on_hover_text(STATUS_NODE_UPTIME);
        ui.label(api.uptime.to_string());

        ui.label(RichText::new("Block Height").underline().color(BONE))
            .on_hover_text(STATUS_NODE_BLOCK_HEIGHT);
        ui.label(api.blockheight.to_string());
        ui.label(RichText::new("Network Difficulty").underline().color(BONE))
            .on_hover_text(STATUS_NODE_DIFFICULTY);
        ui.label(api.difficulty.to_string());
        ui.label(RichText::new("Database size").underline().color(BONE))
            .on_hover_text(STATUS_NODE_DB_SIZE);
        ui.label(api.database_size.to_owned());
        ui.label(RichText::new("Free space").underline().color(BONE))
            .on_hover_text(STATUS_NODE_FREESPACE);
        ui.label(api.free_space.to_owned());
        ui.label(RichText::new("Network Type").underline().color(BONE))
            .on_hover_text(STATUS_NODE_NETTYPE);
        ui.label(api.nettype.to_string());
        ui.label(RichText::new("Outgoing peers").underline().color(BONE))
            .on_hover_text(STATUS_NODE_OUT);
        ui.label(api.outgoing_connections.to_string());
        ui.label(RichText::new("Incoming peers").underline().color(BONE))
            .on_hover_text(STATUS_NODE_IN);
        ui.label(api.incoming_connections.to_string());
        ui.label(RichText::new("Synchronized").underline().color(BONE))
            .on_hover_text(STATUS_NODE_SYNC);
        ui.label(api.synchronized.to_string());
        ui.label(RichText::new("Status").underline().color(BONE))
            .on_hover_text(STATUS_NODE_STATUS);
        ui.label(api.status.to_string());
        drop(api);
    });
}
