use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::helper::xvb::api_url_xmrig;
use crate::helper::xvb::current_controllable_hr;
use crate::miscs::output_console;
use crate::miscs::output_console_without_time;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    helper::{
        p2pool::PubP2poolApi,
        xrig::{update_xmrig_config, xmrig::PubXmrigApi},
        xvb::{nodes::XvbNode, priv_stats::RuntimeMode},
    },
    macros::lock,
    BLOCK_PPLNS_WINDOW_MAIN, BLOCK_PPLNS_WINDOW_MINI, SECOND_PER_BLOCK_P2POOL, XVB_BUFFER,
    XVB_ROUND_DONOR_MEGA_MIN_HR, XVB_ROUND_DONOR_MIN_HR, XVB_ROUND_DONOR_VIP_MIN_HR,
    XVB_ROUND_DONOR_WHALE_MIN_HR, XVB_TIME_ALGO,
};

use super::{priv_stats::RuntimeDonationLevel, PubXvbApi, SamplesAverageHour};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn algorithm(
    client: &Client,
    pub_api: &Arc<Mutex<PubXvbApi>>,
    gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
    gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
    gui_api_xp: &Arc<Mutex<PubXmrigProxyApi>>,
    gui_api_p2pool: &Arc<Mutex<PubP2poolApi>>,
    token_xmrig: &str,
    state_p2pool: &crate::disk::state::P2pool,
    share: u32,
    time_donated: &Arc<Mutex<u32>>,
    rig: &str,
    xp_alive: bool,
) {
    let mut algorithm = Algorithm::new(
        client,
        pub_api,
        gui_api_xvb,
        gui_api_xmrig,
        gui_api_xp,
        gui_api_p2pool,
        token_xmrig,
        state_p2pool,
        share,
        time_donated,
        rig,
        xp_alive,
    );
    algorithm.run().await;
}

#[allow(dead_code)]
pub struct Algorithm<'a> {
    client: &'a Client,
    pub_api: &'a Arc<Mutex<PubXvbApi>>,
    gui_api_xvb: &'a Arc<Mutex<PubXvbApi>>,
    gui_api_xmrig: &'a Arc<Mutex<PubXmrigApi>>,
    gui_api_xp: &'a Arc<Mutex<PubXmrigProxyApi>>,
    gui_api_p2pool: &'a Arc<Mutex<PubP2poolApi>>,
    token_xmrig: &'a str,
    state_p2pool: &'a crate::disk::state::P2pool,
    time_donated: &'a Arc<Mutex<u32>>,
    rig: &'a str,
    xp_alive: bool,
    pub stats: Stats,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Stats {
    share: u32,
    hashrate_xmrig: f32,
    pub target_donation_hashrate: f32,
    xvb_24h_avg: f32,
    xvb_1h_avg: f32,
    address: String,
    runtime_mode: RuntimeMode,
    runtime_donation_level: RuntimeDonationLevel,
    runtime_amount: f64,
    p2pool_total_hashrate: f32,
    avg_last_hour_hashrate: f32,
    p2pool_external_hashrate: f32,
    share_min_hashrate: f32,
    spareable_hashrate: f32,
    spared_time: u32,
    api_url: String,
    msg_xmrig_or_xp: String,
}

impl<'a> Algorithm<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: &'a Client,
        pub_api: &'a Arc<Mutex<PubXvbApi>>,
        gui_api_xvb: &'a Arc<Mutex<PubXvbApi>>,
        gui_api_xmrig: &'a Arc<Mutex<PubXmrigApi>>,
        gui_api_xp: &'a Arc<Mutex<PubXmrigProxyApi>>,
        gui_api_p2pool: &'a Arc<Mutex<PubP2poolApi>>,
        token_xmrig: &'a str,
        state_p2pool: &'a crate::disk::state::P2pool,
        share: u32,
        time_donated: &'a Arc<Mutex<u32>>,
        rig: &'a str,
        xp_alive: bool,
    ) -> Self {
        let hashrate_xmrig = current_controllable_hr(xp_alive, gui_api_xp, gui_api_xmrig);

        let address = state_p2pool.address.clone();

        let runtime_mode = lock!(gui_api_xvb).stats_priv.runtime_mode.clone();
        let runtime_donation_level = lock!(gui_api_xvb)
            .stats_priv
            .runtime_manual_donation_level
            .clone();
        let runtime_amount = lock!(gui_api_xvb).stats_priv.runtime_manual_amount;

        let p2pool_total_hashrate = lock!(gui_api_p2pool).sidechain_ehr;

        let avg_last_hour_hashrate =
            Self::calc_last_hour_avg_hash_rate(&lock!(gui_api_xvb).p2pool_sent_last_hour_samples);
        let mut p2pool_external_hashrate = p2pool_total_hashrate - avg_last_hour_hashrate;
        if p2pool_external_hashrate < 0.0 {
            p2pool_external_hashrate = 0.0;
        }

        let share_min_hashrate = Self::minimum_hashrate_share(
            lock!(gui_api_p2pool).p2pool_difficulty_u64,
            state_p2pool.mini,
            p2pool_external_hashrate,
        );

        let spareable_hashrate = hashrate_xmrig - share_min_hashrate;

        let api_url = api_url_xmrig(xp_alive, true);

        let msg_xmrig_or_xp = (if xp_alive { "XMRig-Proxy" } else { "XMRig" }).to_string();
        info!("xp alive: {:?}", xp_alive);

        let xvb_24h_avg = lock!(pub_api).stats_priv.donor_24hr_avg * 1000.0;
        let xvb_1h_avg = lock!(pub_api).stats_priv.donor_1hr_avg * 1000.0;

        let stats = Stats {
            share,
            hashrate_xmrig,
            xvb_24h_avg,
            xvb_1h_avg,
            address,
            target_donation_hashrate: f32::default(),
            runtime_mode,
            runtime_donation_level,
            runtime_amount,
            p2pool_total_hashrate,
            avg_last_hour_hashrate,
            p2pool_external_hashrate,
            share_min_hashrate,
            spareable_hashrate,
            spared_time: u32::default(),
            api_url,
            msg_xmrig_or_xp,
        };

        let mut new_instance = Self {
            client,
            pub_api,
            gui_api_xvb,
            gui_api_xmrig,
            gui_api_xp,
            gui_api_p2pool,
            token_xmrig,
            state_p2pool,
            time_donated,
            rig,
            xp_alive,
            stats,
        };

        new_instance.stats.target_donation_hashrate = new_instance.get_target_donation_hashrate();

        new_instance.stats.spared_time = Self::get_spared_time(
            new_instance.stats.target_donation_hashrate,
            new_instance.stats.hashrate_xmrig,
        );

        new_instance
    }

    fn is_share_fulfilled(&self) -> bool {
        let is_criteria_fulfilled = self.stats.share > 0;

        info!(
            "Algorithm | shares({}) > 0 : {}",
            self.stats.share, is_criteria_fulfilled,
        );

        is_criteria_fulfilled
    }

    fn is_xvb_24h_fulfilled(&self) -> bool {
        let is_criteria_fulfilled = self.stats.xvb_24h_avg > self.stats.target_donation_hashrate;
        info!(
            "Algorithm | xvb_24h_avg({}) > target_donation_hashrate({}) : {}",
            self.stats.xvb_24h_avg, self.stats.target_donation_hashrate, is_criteria_fulfilled
        );
        is_criteria_fulfilled
    }

    async fn target_p2pool_node(&self) {
        if lock!(self.gui_api_xvb).current_node != Some(XvbNode::P2pool) {
            info!(
                "Algorithm | request {} to mine on p2pool",
                self.stats.msg_xmrig_or_xp
            );

            if let Err(err) = update_xmrig_config(
                self.client,
                &self.stats.api_url,
                self.token_xmrig,
                &XvbNode::P2pool,
                &self.stats.address,
                self.rig,
            )
            .await
            {
                warn!(
                    "Algorithm | Failed request HTTP API {}",
                    self.stats.msg_xmrig_or_xp
                );
                output_console(
                    &mut lock!(self.gui_api_xvb).output,
                    &format!(
                        "Failure to update {} config with HTTP API.\nError: {}",
                        self.stats.msg_xmrig_or_xp, err
                    ),
                    crate::helper::ProcessName::Xvb,
                );
            } else {
                info!(
                    "Algorithm | {} mining on p2pool pool",
                    self.stats.msg_xmrig_or_xp
                );
            }
        }
    }

    async fn target_xvb_node(&self) {
        let node = lock!(self.gui_api_xvb).stats_priv.node;

        info!(
            "Algorithm | request {} to mine on XvB",
            self.stats.msg_xmrig_or_xp
        );

        if lock!(self.gui_api_xvb).current_node.is_none()
            || lock!(self.gui_api_xvb)
                .current_node
                .as_ref()
                .is_some_and(|n| n == &XvbNode::P2pool)
        {
            if let Err(err) = update_xmrig_config(
                self.client,
                &self.stats.api_url,
                self.token_xmrig,
                &node,
                &self.stats.address,
                self.rig,
            )
            .await
            {
                // show to console error about updating xmrig config
                warn!(
                    "Algorithm | Failed request HTTP API {}",
                    self.stats.msg_xmrig_or_xp
                );
                output_console(
                    &mut lock!(self.gui_api_xvb).output,
                    &format!(
                        "Failure to update {} config with HTTP API.\nError: {}",
                        self.stats.msg_xmrig_or_xp, err
                    ),
                    crate::helper::ProcessName::Xvb,
                );
            } else {
                if self.xp_alive {
                    lock!(self.gui_api_xp).node = node.to_string();
                } else {
                    lock!(self.gui_api_xmrig).node = node.to_string();
                }
                info!(
                    "Algorithm | {} mining on XvB pool",
                    self.stats.msg_xmrig_or_xp
                );
            }
        }
    }

    async fn send_all_p2pool(&self) {
        self.target_p2pool_node().await;

        info!(
            "Algorithm | algo sleep for {} seconds while mining on P2pool",
            XVB_TIME_ALGO
        );
        sleep(Duration::from_secs(XVB_TIME_ALGO.into())).await;

        lock!(self.gui_api_xvb)
            .p2pool_sent_last_hour_samples
            .0
            .push_back(lock!(self.gui_api_xmrig).hashrate_raw_15m);
        lock!(self.gui_api_xvb)
            .xvb_sent_last_hour_samples
            .0
            .push_back(0.0);
    }

    async fn send_all_xvb(&self) {
        self.target_xvb_node().await;

        info!(
            "Algorithm | algo sleep for {} seconds while mining on XvB",
            XVB_TIME_ALGO
        );
        sleep(Duration::from_secs(XVB_TIME_ALGO.into())).await;

        lock!(self.gui_api_xvb)
            .p2pool_sent_last_hour_samples
            .0
            .push_back(lock!(self.gui_api_xmrig).hashrate_raw_15m);
        lock!(self.gui_api_xvb)
            .xvb_sent_last_hour_samples
            .0
            .push_back(0.0);
    }

    async fn sleep_then_update_node_xmrig(&self) {
        info!(
            "Algorithm | algo sleep for {} seconds while mining on P2pool",
            XVB_TIME_ALGO - self.stats.spared_time
        );
        sleep(Duration::from_secs(
            (XVB_TIME_ALGO - self.stats.spared_time).into(),
        ))
        .await;

        // only update xmrig config if it is actually mining.
        info!("Algorithm | request xmrig to mine on XvB");

        self.target_xvb_node().await;

        // will not quit the process until it is really done.
        // xvb process watch this algo handle to see if process is finished or not.

        info!(
            "Algorithm | algo sleep for {} seconds while mining on P2pool",
            self.stats.spared_time
        );
        sleep(Duration::from_secs(self.stats.spared_time.into())).await;

        lock!(self.gui_api_xvb)
            .p2pool_sent_last_hour_samples
            .0
            .push_back(
                self.stats.hashrate_xmrig
                    * ((XVB_TIME_ALGO as f32 - self.stats.spared_time as f32)
                        / XVB_TIME_ALGO as f32),
            );
        lock!(self.gui_api_xvb)
            .xvb_sent_last_hour_samples
            .0
            .push_back(
                self.stats.hashrate_xmrig * (self.stats.spared_time as f32 / XVB_TIME_ALGO as f32),
            );
    }

    pub fn get_target_donation_hashrate(&self) -> f32 {
        match self.stats.runtime_mode {
            RuntimeMode::Auto => self.get_auto_mode_target_donation_hashrate(),
            RuntimeMode::Hero => self.get_hero_mode_target_donation_hashrate(),
            RuntimeMode::ManualXvb => {
                info!(
                    "Algorithm | ManualXvBMode target_donation_hashrate=runtime_amount({}H/s)",
                    self.stats.runtime_amount
                );

                self.stats.runtime_amount as f32
            }
            RuntimeMode::ManualP2pool => {
                let target_donation_hashrate =
                    self.stats.hashrate_xmrig - (self.stats.runtime_amount as f32);

                info!("Algorithm | ManualP2poolMode target_donation_hashrate({})=hashrate_xmrig({})-runtime_amount({})",
                target_donation_hashrate,
                self.stats.hashrate_xmrig,
                self.stats.runtime_amount);

                target_donation_hashrate
            }
            RuntimeMode::ManualDonationLevel => {
                let target_donation_hashrate = self.stats.runtime_donation_level.get_hashrate();

                info!("Algorithm | ManualDonationLevelMode target_donation_hashrate({})={:#?}.get_hashrate()",
                target_donation_hashrate,
                self.stats.runtime_donation_level);

                target_donation_hashrate
            }
        }
    }

    fn get_auto_mode_target_donation_hashrate(&self) -> f32 {
        let donation_level = match self.stats.spareable_hashrate {
            x if x > (XVB_ROUND_DONOR_MEGA_MIN_HR as f32) => Some(RuntimeDonationLevel::DonorMega),
            x if x > (XVB_ROUND_DONOR_WHALE_MIN_HR as f32) => {
                Some(RuntimeDonationLevel::DonorWhale)
            }
            x if x > (XVB_ROUND_DONOR_VIP_MIN_HR as f32) => Some(RuntimeDonationLevel::DonorVIP),
            x if x > (XVB_ROUND_DONOR_MIN_HR as f32) => Some(RuntimeDonationLevel::Donor),
            _ => None,
        };

        info!(
            "Algorithm | AutoMode target_donation_level detected ({:#?})",
            donation_level
        );

        let target_donation_hashrate = if let Some(level) = donation_level {
            level.get_hashrate()
        } else {
            0.0
        };

        info!(
            "Algorithm | AutoMode target_donation_hashrate ({})",
            target_donation_hashrate
        );

        target_donation_hashrate
    }

    fn get_hero_mode_target_donation_hashrate(&self) -> f32 {
        info!(
            "Algorithm | HeroMode target_donation_hashrate=spareable_hashrate({})",
            self.stats.spareable_hashrate
        );

        self.stats.spareable_hashrate
    }

    // push new value into samples before executing this calcul
    fn calc_last_hour_avg_hash_rate(samples: &SamplesAverageHour) -> f32 {
        samples.0.iter().sum::<f32>() / samples.0.len() as f32
    }

    fn minimum_hashrate_share(difficulty: u64, mini: bool, p2pool_external_hashrate: f32) -> f32 {
        let pws = if mini {
            BLOCK_PPLNS_WINDOW_MINI
        } else {
            BLOCK_PPLNS_WINDOW_MAIN
        };
        let mut minimum_hr = ((difficulty / (pws * SECOND_PER_BLOCK_P2POOL)) as f32 * XVB_BUFFER)
            - p2pool_external_hashrate;

        info!("Algorithm | (difficulty({}) / (window pplns blocks({}) * seconds per p2pool block({})) * BUFFER({})) - outside HR({}H/s) = minimum HR({}H/s) to keep a share.",
         difficulty,
         pws,
         SECOND_PER_BLOCK_P2POOL,
         XVB_BUFFER,
         p2pool_external_hashrate,
         minimum_hr);

        if minimum_hr.is_sign_negative() {
            info!("Algorithm | if minimum HR is negative, it is 0.");
            minimum_hr = 0.0;
        }

        minimum_hr
    }

    async fn fulfill_share(&self) {
        output_console(
            &mut lock!(self.gui_api_xvb).output,
            "There are no shares in p2pool. Sending all hashrate to p2pool!",
            crate::helper::ProcessName::Xvb,
        );

        info!("Algorithm | There are no shares in p2pool. Sending all hashrate to p2pool!");

        self.send_all_p2pool().await
    }

    async fn fulfill_xvb_24_avg(&self) {
        output_console(
            &mut lock!(self.gui_api_xvb).output,
            "24H avg XvB target not achieved. Sending all hashrate to XvB!",
            crate::helper::ProcessName::Xvb,
        );

        info!("Algorithm | 24H avg XvB target not achieved. Sending all hashrate to XvB!");

        *lock!(self.time_donated) = XVB_TIME_ALGO;

        self.send_all_xvb().await
    }

    async fn fulfill_normal_cycles(&self) {
        output_console(
            &mut lock!(self.gui_api_xvb).output,
            &format!(
                "There is a share in p2pool and 24H avg XvB is achieved. Sending {} seconds to XvB!",
                self.stats.spared_time
            ),
            crate::helper::ProcessName::Xvb,
        );

        info!("Algorithm | There is a share in p2pool and 24H avg XvB is achieved. Sending seconds {} to XvB!", self.stats.spared_time);

        *lock!(self.time_donated) = self.stats.spared_time;

        self.target_p2pool_node().await;
        self.sleep_then_update_node_xmrig().await;
    }

    pub async fn run(&mut self) {
        output_console(
            &mut lock!(self.gui_api_xvb).output,
            "Algorithm of HR distribution started for the next 10 minutes.",
            crate::helper::ProcessName::Xvb,
        );

        info!("Algorithm | Starting...");
        info!("Algorithm | {:#?}", self.stats);

        if !self.is_share_fulfilled() {
            self.fulfill_share().await
        } else if !self.is_xvb_24h_fulfilled() {
            self.fulfill_xvb_24_avg().await
        } else {
            self.fulfill_normal_cycles().await
        }

        output_console_without_time(
            &mut lock!(self.gui_api_xvb).output,
            "",
            crate::helper::ProcessName::Xvb,
        )
    }

    fn get_spared_time(target_donation_hashrate: f32, hashrate_xmrig: f32) -> u32 {
        let spared_time = target_donation_hashrate / hashrate_xmrig * (XVB_TIME_ALGO as f32);

        info!("Algorithm | Calculating... spared_time({}seconds)=target_donation_hashrate({})/hashrate_xmrig({})*XVB_TIME_ALGO({})",
        spared_time,
        target_donation_hashrate,
        hashrate_xmrig,
        XVB_TIME_ALGO);

        spared_time as u32
    }
}
