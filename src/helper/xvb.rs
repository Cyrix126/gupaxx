use anyhow::{bail, Result};
use bytes::Bytes;
use derive_more::Display;
use hyper::client::HttpConnector;
use hyper::{Client, Request, StatusCode};
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};
use readable::up::Uptime;
use serde::Deserialize;
use std::fmt::Write;
use std::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use tokio::spawn;
use tokio::time::sleep_until;

use crate::components::node::{GetInfo, TIMEOUT_NODE_PING};
use crate::helper::xmrig::PrivXmrigApi;
use crate::utils::constants::{
    BLOCK_PPLNS_WINDOW_MAIN, BLOCK_PPLNS_WINDOW_MINI, GUPAX_VERSION_UNDERSCORE,
    SECOND_PER_BLOCK_P2POOL, XMRIG_CONFIG_URI, XVB_BUFFER, XVB_NODE_EU, XVB_NODE_NA, XVB_NODE_PORT,
    XVB_NODE_RPC, XVB_PUBLIC_ONLY, XVB_ROUND_DONOR_MEGA_MIN_HR, XVB_ROUND_DONOR_MIN_HR,
    XVB_ROUND_DONOR_VIP_MIN_HR, XVB_ROUND_DONOR_WHALE_MIN_HR, XVB_TIME_ALGO, XVB_URL,
};
use crate::{
    helper::{ProcessSignal, ProcessState},
    utils::{
        constants::{HORI_CONSOLE, XVB_URL_PUBLIC_API},
        macros::{lock, lock2, sleep},
    },
};

use super::p2pool::PubP2poolApi;
use super::xmrig::PubXmrigApi;
use super::{Helper, Process};

impl Helper {
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_xvb(helper: &Arc<Mutex<Self>>) {
        info!("XvB | Attempting to stop...");
        lock2!(helper, xvb).signal = ProcessSignal::Stop;
        lock2!(helper, xvb).state = ProcessState::Middle;
    }
    pub fn restart_xvb(
        helper: &Arc<Mutex<Self>>,
        state_xvb: &crate::disk::state::Xvb,
        state_p2pool: &crate::disk::state::P2pool,
        state_xmrig: &crate::disk::state::Xmrig,
    ) {
        info!("XvB | Attempting to restart...");
        lock2!(helper, xvb).signal = ProcessSignal::Restart;
        lock2!(helper, xvb).state = ProcessState::Middle;
        let helper = helper.clone();
        let state_xvb = state_xvb.clone();
        let state_p2pool = state_p2pool.clone();
        let state_xmrig = state_xmrig.clone();
        // This thread lives to wait, start xmrig then die.
        thread::spawn(move || {
            while lock2!(helper, xvb).state != ProcessState::Waiting {
                warn!("XvB | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("XvB | Old process seems dead, starting new one!");
            Self::start_xvb(&helper, &state_xvb, &state_p2pool, &state_xmrig);
        });
        info!("XMRig | Restart ... OK");
    }
    pub fn start_xvb(
        helper: &Arc<Mutex<Self>>,
        state_xvb: &crate::disk::state::Xvb,
        state_p2pool: &crate::disk::state::P2pool,
        state_xmrig: &crate::disk::state::Xmrig,
    ) {
        info!("XvB | setting state to Middle");
        lock2!(helper, xvb).state = ProcessState::Middle;
        info!("XvB | cloning helper arc fields");
        let gui_api = Arc::clone(&lock!(helper).gui_api_xvb);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xvb);
        let process = Arc::clone(&lock!(helper).xvb);
        // needed to see if it is alive. For XvB process to function completely, p2pool node must be alive to check the shares in the pplns window.
        let process_p2pool = Arc::clone(&lock!(helper).p2pool);
        let gui_api_p2pool = Arc::clone(&lock!(helper).gui_api_p2pool);
        let process_xmrig = Arc::clone(&lock!(helper).xmrig);
        let gui_api_xmrig = Arc::clone(&lock!(helper).gui_api_xmrig);
        info!("XvB | cloning of state");
        // Reset before printing to output.
        // Need to reset because values of stats would stay otherwise which could bring confusion even if panel is with a disabled theme.
        info!("XvB | resetting pub and gui");
        *lock!(pub_api) = PubXvbApi::new();
        *lock!(gui_api) = PubXvbApi::new();
        // 2. Set process state
        info!("XvB | Setting process state...");
        {
            let mut lock = lock!(process);
            lock.state = ProcessState::Middle;
            lock.signal = ProcessSignal::None;
            lock.start = Instant::now();
        }
        // verify if token and address are existent on XvB server
        let state_xvb = state_xvb.clone();
        let state_p2pool = state_p2pool.clone();
        let state_xmrig = state_xmrig.clone();

        info!("XvB | spawn watchdog");
        thread::spawn(move || {
            Self::spawn_xvb_watchdog(
                gui_api,
                pub_api,
                process,
                &state_xvb,
                &state_p2pool,
                &state_xmrig,
                gui_api_p2pool,
                process_p2pool,
                gui_api_xmrig,
                process_xmrig,
            );
        });
    }
    #[tokio::main]
    async fn spawn_xvb_watchdog(
        gui_api: Arc<Mutex<PubXvbApi>>,
        pub_api: Arc<Mutex<PubXvbApi>>,
        process: Arc<Mutex<Process>>,
        state_xvb: &crate::disk::state::Xvb,
        state_p2pool: &crate::disk::state::P2pool,
        state_xmrig: &crate::disk::state::Xmrig,
        gui_api_p2pool: Arc<Mutex<PubP2poolApi>>,
        process_p2pool: Arc<Mutex<Process>>,
        gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,
        process_xmrig: Arc<Mutex<Process>>,
    ) {
        // client for http and one for https, both will be valid for the thread scope.
        let https = HttpsConnector::new();
        let client_https = hyper::Client::builder().build(https);
        // client http should be created only if process completely started, because it's not useful otherwise, but it would possibly be unitiliazed.
        let client_http =
            Arc::new(hyper::Client::builder().build(hyper::client::HttpConnector::new()));
        info!("XvB | verify address and token");
        if let Err(err) =
            XvbPrivStats::request_api(&client_https, &state_p2pool.address, &state_xvb.token).await
        {
            // send to console: token non existent for address on XvB server
            warn!("Xvb | Start ... Partially failed because token and associated address are not existent on XvB server: {}\n", err);
            output_console(&gui_api, &format!("Token and associated address are not valid on XvB API.\nCheck if you are registered.\nError: {}", err));
            lock!(process).state = ProcessState::NotMining;
        }
        info!("XvB | verify p2pool node");
        if !lock!(process_p2pool).is_alive() {
            // send to console: p2pool process is not running
            warn!("Xvb | Start ... Partially failed because P2pool instance is not running.");
            output_console(
                &gui_api,
                "P2pool process is not running.\nCheck the P2pool Tab",
            );
            lock!(process).state = ProcessState::Syncing;
        }

        if !lock!(process_xmrig).is_alive() {
            // send to console: p2pool process is not running
            warn!("Xvb | Start ... Partially failed because Xmrig instance is not running.");
            // output the error to console
            output_console(
                &gui_api,
                "XMRig process is not running.\nCheck the Xmrig Tab.",
            );
            lock!(process).state = ProcessState::Syncing;
        }
        info!("XvB | print to console state");
        if lock!(process).state != ProcessState::Middle {
            output_console(
                &gui_api,
                &["XvB partially started.\n", XVB_PUBLIC_ONLY].concat(),
            );
        } else {
            info!("XvB Fully started");
            lock!(process).state = ProcessState::Alive;

            let pub_api_c = pub_api.clone();
            let client_http_c = client_http.clone();
            let process_c = process.clone();
            let gui_api_c = gui_api.clone();
            // will check which pool to use, will send NotMining if
            spawn(async move {
                XvbNode::update_fastest_node(&client_http_c, &pub_api_c, &gui_api_c, &process_c)
                    .await;
            });
            output_console(
                &gui_api,
                "Algorithm of distribution of HR will wait 15 minutes for Xmrig average HR data.",
            );
        }
        // see how many shares are found at p2pool node only if XvB is started successfully. If it wasn't, maybe P2pool is node not running.
        let mut old_shares = if lock!(process).state == ProcessState::Alive {
            // a loop until the value is some to let p2pool work and get first value.
            loop {
                if let Some(s) = lock!(gui_api_p2pool).shares_found {
                    break s;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        } else {
            // if Syncing state, this value is not needed
            0
        };
        // let mut old_shares = 0;
        let mut time_last_share: Option<Instant> = None;
        let start = lock!(process).start;
        let mut start_algorithm = tokio::time::Instant::now();
        info!("XvB | Entering watchdog mode... woof!");
        loop {
            debug!("XvB Watchdog | ----------- Start of loop -----------");
            // if address and token valid, verify if p2pool and xmrig are running, else XvBmust be reloaded with another token/address to start verifying the other process.
            if lock!(process).state != ProcessState::NotMining {
                // verify if p2pool node and xmrig are running
                if lock!(process_p2pool).is_alive() && lock!(process_xmrig).is_alive() {
                    // verify if state is to changed
                    if lock!(process).state == ProcessState::Syncing {
                        info!("XvB | started this time with p2pool and xmrig");
                        *lock!(pub_api) = PubXvbApi::new();
                        *lock!(gui_api) = PubXvbApi::new();
                        lock!(process).state = ProcessState::Alive;
                        output_console(
                            &gui_api,
                            "XvB is now started because p2pool and xmrig came online.",
                        );
                    }
                } else {
                    // verify if the state is changing because p2pool is not alive anymore.
                    if lock!(process).state != ProcessState::Syncing {
                        info!("XvB | stopped partially because p2pool is not alive anymore.");
                        *lock!(pub_api) = PubXvbApi::new();
                        *lock!(gui_api) = PubXvbApi::new();
                        lock!(process).state = ProcessState::Syncing;
                        output_console(
                            &gui_api,
                            "XvB is now partially stopped because p2pool node or xmrig came offline.\nCheck P2pool and Xmrig Tabs",
                        );
                    }
                }
            }

            // Set timer
            let now = Instant::now();
            // check signal
            debug!("XvB | check signal");
            if signal_interrupt(
                process.clone(),
                start,
                &client_http,
                &pub_api,
                &gui_api,
                &gui_api_xmrig,
                state_p2pool,
                state_xmrig,
            ) {
                break;
            }
            // verify if
            // Send an HTTP API request only if one minute is passed since the last.
            // if since is 0, send request because it's the first time.
            let since = lock!(gui_api).tick;
            if since >= 60 || since == 0 {
                debug!("XvB Watchdog | Attempting HTTP public API request...");
                match XvbPubStats::request_api(&client_https).await {
                    Ok(new_data) => {
                        debug!("XvB Watchdog | HTTP API request OK");
                        lock!(&pub_api).stats_pub = new_data;
                    }
                    Err(err) => {
                        warn!(
                            "XvB Watchdog | Could not send HTTP API request to: {}\n:{}",
                            XVB_URL_PUBLIC_API, err
                        );
                        // output the error to console
                        output_console(
                            &gui_api,
                            &format!(
                                "Failure to retrieve public stats from {}",
                                XVB_URL_PUBLIC_API
                            ),
                        );
                        lock!(process).state = ProcessState::Failed;
                        break;
                    }
                }
                // only if private API is accessible, NotMining here means that the token and address is not registered on the XvB website and Syncing means P2pool node is not running and so that some information for private info are not available.
                if lock!(process).state == ProcessState::Alive {
                    debug!("XvB Watchdog | Attempting HTTP private API request...");
                    // reload private stats
                    match XvbPrivStats::request_api(
                        &client_https,
                        &state_p2pool.address,
                        &state_xvb.token,
                    )
                    .await
                    {
                        Ok(b) => {
                            debug!("XvB Watchdog | HTTP API request OK");
                            let new_data = match serde_json::from_slice::<XvbPrivStats>(&b) {
                                Ok(data) => data,
                                Err(e) => {
                                    warn!("XvB Watchdog | Data provided from private API is not deserializ-able.Error: {}", e);
                                    output_console(&gui_api, &format!("XvB Watchdog | Data provided from private API is not deserializ-able.Error: {}", e));
                                    break;
                                }
                            };
                            lock!(&pub_api).stats_priv = new_data;
                        }
                        Err(err) => {
                            warn!(
                            "XvB Watchdog | Could not send HTTP private API request to: {}\n:{}",
                            XVB_URL, err
                        );
                            output_console(
                                &gui_api,
                                &format!("Failure to retrieve private stats from {}", XVB_URL),
                            );
                            lock!(process).state = ProcessState::Failed;
                            break;
                        }
                    }
                    // check if share is in pplns window
                    // p2pool local api show only found shares and not current shares. So we need to keep track of the time
                    // the height of p2pool would be nicer but the p2pool api doesn't show it.
                    let (share, new_time) = lock!(gui_api_p2pool)
                        .is_share_present_in_ppplns_window(
                            &mut old_shares,
                            time_last_share,
                            state_p2pool.mini,
                        );
                    if let Some(n) = new_time {
                        time_last_share = Some(n);
                    }

                    //     // verify in which round type we are
                    let round = if share {
                        let stats_priv = &lock!(pub_api).stats_priv;
                        match (
                            stats_priv.donor_1hr_avg as u32,
                            stats_priv.donor_24hr_avg as u32,
                        ) {
                            x if x.0 > XVB_ROUND_DONOR_MEGA_MIN_HR
                                && x.1 > XVB_ROUND_DONOR_MEGA_MIN_HR =>
                            {
                                Some(XvbRound::DonorMega)
                            }
                            x if x.0 > XVB_ROUND_DONOR_WHALE_MIN_HR
                                && x.1 > XVB_ROUND_DONOR_WHALE_MIN_HR =>
                            {
                                Some(XvbRound::DonorWhale)
                            }
                            x if x.0 > XVB_ROUND_DONOR_VIP_MIN_HR
                                && x.1 > XVB_ROUND_DONOR_VIP_MIN_HR =>
                            {
                                Some(XvbRound::DonorVip)
                            }
                            x if x.0 > XVB_ROUND_DONOR_MIN_HR && x.1 > XVB_ROUND_DONOR_MIN_HR => {
                                Some(XvbRound::Donor)
                            }
                            (_, _) => Some(XvbRound::Vip),
                        }
                    } else {
                        None
                    };
                    // refresh the round we participate
                    lock!(&pub_api).stats_priv.round_participate = round;

                    // verify if we are the winner of the current round
                    if &lock!(pub_api).stats_pub.winner
                        == Helper::head_tail_of_monero_address(&state_p2pool.address).as_str()
                    {
                        lock!(pub_api).stats_priv.win_current = true
                    }

                    // if 10 minutes passed since last check
                    // the first 15 minutes, the HR of xmrig will be 0.0, so xmrig will always mine on p2pool for 15m.
                    if start_algorithm.elapsed() >= Duration::from_secs(XVB_TIME_ALGO.into()) {
                        info!("Xvb Process | Algorithm is started");
                        // the time that takes the algorithm do decide the next ten minutes could means less p2pool mining. It is solved by the buffer and spawning requests.
                        start_algorithm = tokio::time::Instant::now();
                        // request XMrig to mine on P2pool
                        info!("Xvb Process | request to mine on p2pool");
                        let client_http_c = client_http.clone();
                        let gui_api_c = gui_api.clone();
                        let token_xmrig = Arc::new(state_xmrig.token.clone());
                        let token_xmrig_c = token_xmrig.clone();
                        let address = Arc::new(state_p2pool.address.clone());
                        let address_c = address.clone();
                        let gui_api_xmrig_c = gui_api_xmrig.clone();
                        spawn(async move {
                            if let Err(err) = PrivXmrigApi::update_xmrig_config(
                                &client_http_c,
                                XMRIG_CONFIG_URI,
                                &token_xmrig_c,
                                &XvbNode::P2pool,
                                &address_c,
                                &gui_api_xmrig_c,
                            )
                            .await
                            {
                                output_console(
                                    &gui_api_c,
                                    &format!(
                                        "Failure to update xmrig config with HTTP API.\nError: {}",
                                        err
                                    ),
                                );
                            } else {
                                output_console(&gui_api_c, "Algorithm of distribution HR started for the next ten minutes.\nMining on local p2pool node.");
                            }
                        });

                        // if share is in PW,
                        if share {
                            info!("Xvb Process | Algorithm share is in current window");
                            // calcul minimum HR

                            output_console(
                                &gui_api,
                                "At least one share is in current PPLNS window.",
                            );
                            let hr = lock!(gui_api_xmrig).hashrate_raw_15m;
                            let min_hr = Helper::minimum_hashrate_share(
                                lock!(gui_api_p2pool).p2pool_difficulty_u64,
                                state_p2pool.mini,
                            );
                            info!("Xvb Process | hr {}, min_hr: {} ", hr, min_hr);
                            output_console(
                                &gui_api,
                                &format!("You'r HR from Xmrig is {}, minimum required HR to keep a share in PPLNS window is {}", hr, min_hr),
                            );

                            // calculate how much time can be spared
                            let mut spared_time = Helper::time_that_could_be_spared(hr, min_hr);
                            if spared_time > 0 {
                                // if not hero option
                                if !state_xvb.hero {
                                    // calculate how much time needed to be spared to be in most round type minimum HR + buffer
                                    spared_time = Helper::minimum_time_for_highest_accessible_round(
                                        spared_time,
                                        hr,
                                    );
                                }
                                info!("Xvb Process | spared time {} ", spared_time);
                                output_console(
                                    &gui_api,
                                    &format!(
                                        " {} seconds of HR will be donated to the raffle.",
                                        spared_time
                                    ),
                                );
                                // sleep 10m less spared time then request XMrig to mine on XvB
                                let was_instant = start_algorithm;
                                let gui_api_c = gui_api.clone();
                                let client_http_c = client_http.clone();
                                let gui_api_xmrig_c = gui_api_xmrig.clone();
                                spawn(async move {
                                    Helper::sleep_then_update_node_xmrig(
                                        &was_instant,
                                        spared_time,
                                        &client_http_c,
                                        XMRIG_CONFIG_URI,
                                        &token_xmrig,
                                        &address,
                                        gui_api_c,
                                        gui_api_xmrig_c,
                                    )
                                    .await
                                });
                            }
                        } else {
                            output_console(&gui_api, "No share in the current PPLNS Window !");
                        }
                    }
                    // instant saved for next check
                    // fi
                }

                lock!(gui_api).tick = 0;
            }

            lock!(gui_api).tick += 1;
            // Reset stats before loop

            // Sleep (only if 900ms hasn't passed)
            let elapsed = now.elapsed().as_millis();
            // Since logic goes off if less than 1000, casting should be safe
            if elapsed < 900 {
                let sleep = (900 - elapsed) as u64;
                debug!("XvB Watchdog | END OF LOOP - Sleeping for [{}]s...", sleep);
                std::thread::sleep(std::time::Duration::from_millis(sleep))
            } else {
                debug!("XMRig Watchdog | END OF LOOP - Not sleeping!");
            }
        }
    }
    fn minimum_hashrate_share(difficulty: u64, mini: bool) -> f32 {
        let pws = if mini {
            BLOCK_PPLNS_WINDOW_MINI
        } else {
            BLOCK_PPLNS_WINDOW_MAIN
        };
        (difficulty / (pws * SECOND_PER_BLOCK_P2POOL)) as f32 * XVB_BUFFER
    }
    fn time_that_could_be_spared(hr: f32, min_hr: f32) -> u32 {
        // percent of time minimum
        let minimum_time_required_on_p2pool = XVB_TIME_ALGO as f32 / (hr / min_hr);
        let spared_time = XVB_TIME_ALGO as f32 - minimum_time_required_on_p2pool;
        // if less than 10 seconds, XMRig could hardly have the time to mine anything.
        if spared_time >= 10f32 {
            return spared_time as u32;
        }
        0
    }
    fn minimum_time_for_highest_accessible_round(st: u32, hr: f32) -> u32 {
        let hr_for_xvb = ((st as f32 / XVB_TIME_ALGO as f32) * hr) as u32;
        match hr_for_xvb {
            x if x > XVB_ROUND_DONOR_MEGA_MIN_HR => x - XVB_ROUND_DONOR_MEGA_MIN_HR,
            x if x > XVB_ROUND_DONOR_WHALE_MIN_HR => x - XVB_ROUND_DONOR_WHALE_MIN_HR,
            x if x > XVB_ROUND_DONOR_VIP_MIN_HR => x - XVB_ROUND_DONOR_VIP_MIN_HR,
            x if x > XVB_ROUND_DONOR_MIN_HR => x - XVB_ROUND_DONOR_MIN_HR,
            _ => 0,
        }
    }
    async fn sleep_then_update_node_xmrig(
        was_instant: &tokio::time::Instant,
        spared_time: u32,
        client: &Client<HttpConnector>,
        api_uri: &str,
        token_xmrig: &str,
        address: &str,
        gui_api_xvb: Arc<Mutex<PubXvbApi>>,
        gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,
    ) {
        let node = lock!(gui_api_xvb).stats_priv.node.clone();
        info!("Xvb Process | for now mine on p2pol ");
        info!("Xvb Process | spared time {} ", spared_time);
        output_console(
            &gui_api_xvb,
            &format!(
                "Still mining on P2pool node for {} seconds",
                XVB_TIME_ALGO - spared_time
            ),
        );
        sleep_until(*was_instant + Duration::from_secs((XVB_TIME_ALGO - spared_time) as u64)).await;
        if let Err(err) = PrivXmrigApi::update_xmrig_config(
            client,
            api_uri,
            token_xmrig,
            &node,
            address,
            &gui_api_xmrig,
        )
        .await
        {
            // show to console error about updating xmrig config
            output_console(
                &gui_api_xvb,
                &format!(
                    "Failure to update xmrig config with HTTP API.\nError: {}",
                    err
                ),
            );
        } else {
            info!("Xvb Process | mining on XvB pool");
            output_console(
                &gui_api_xvb,
                &format!(
                    "Now donating to the XvB raffle for the rest of the {} minutes.",
                    XVB_TIME_ALGO / 60
                ),
            );
        }
    }
}
//---------------------------------------------------------------------------------------------------- Public XvB API
use serde_this_or_that::as_u64;
#[derive(Debug, Clone, Default)]
pub struct PubXvbApi {
    pub output: String,
    pub uptime: u64,
    pub tick: u8,
    pub tick_distribute_hr: u16,
    pub stats_pub: XvbPubStats,
    pub stats_priv: XvbPrivStats,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct XvbPubStats {
    pub time_remain: u32, // remaining time of round in minutes
    pub bonus_hr: f64,
    pub donate_hr: f64,      // donated hr from all donors
    pub donate_miners: u32,  // numbers of donors
    pub donate_workers: u32, // numbers of workers from donors
    pub players: u32,
    pub players_round: u32,
    pub winner: String,
    pub share_effort: String,
    pub block_reward: String,
    pub round_type: XvbRound,
    #[serde(deserialize_with = "as_u64")]
    pub block_height: u64,
    pub block_hash: String,
    #[serde(deserialize_with = "as_u64")]
    pub roll_winner: u64,
    #[serde(deserialize_with = "as_u64")]
    pub roll_round: u64,
    pub reward_yearly: Vec<f64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct XvbPrivStats {
    pub fails: u8,
    pub donor_1hr_avg: f32,
    pub donor_24hr_avg: f32,
    #[serde(skip)]
    pub win_current: bool,
    #[serde(skip)]
    pub round_participate: Option<XvbRound>,
    #[serde(skip)]
    pub node: XvbNode,
}

impl XvbPubStats {
    #[inline]
    // Send an HTTP request to XvB's API, serialize it into [Self] and return it
    async fn request_api(
        client: &hyper::Client<HttpsConnector<HttpConnector>>,
    ) -> std::result::Result<Self, anyhow::Error> {
        let request = hyper::Request::builder()
            .method("GET")
            .uri(XVB_URL_PUBLIC_API)
            .body(hyper::Body::empty())?;
        let response =
            tokio::time::timeout(std::time::Duration::from_secs(8), client.request(request))
                .await?;
        // let response = client.request(request).await;

        let body = hyper::body::to_bytes(response?.body_mut()).await?;
        Ok(serde_json::from_slice::<Self>(&body)?)
    }
}
impl XvbPrivStats {
    pub async fn request_api(
        client: &hyper::Client<HttpsConnector<HttpConnector>>,
        address: &str,
        token: &str,
    ) -> Result<Bytes> {
        if let Ok(request) = hyper::Request::builder()
            .method("GET")
            .uri(format!(
                "{}/cgi-bin/p2pool_bonus_history_api.cgi?address={}&token={}",
                XVB_URL, address, token
            ))
            .body(hyper::Body::empty())
        {
            match client.request(request).await {
                Ok(mut resp) => match resp.status() {
                    StatusCode::OK => Ok(hyper::body::to_bytes(resp.body_mut()).await?),
                    StatusCode::UNPROCESSABLE_ENTITY => {
                        bail!("the token is invalid for this xmr address.")
                    }
                    _ => bail!("The status of the response is not expected"),
                },
                Err(err) => {
                    bail!("error from response: {}", err)
                }
            }
        } else {
            bail!("request could not be build")
        }
    }
}

#[derive(Debug, Clone, Default, Display, Deserialize)]
pub enum XvbRound {
    #[default]
    #[display(fmt = "VIP")]
    #[serde(alias = "vip")]
    Vip,
    #[serde(alias = "donor")]
    Donor,
    #[display(fmt = "VIP Donor")]
    #[serde(alias = "donor_vip")]
    DonorVip,
    #[display(fmt = "Whale Donor")]
    #[serde(alias = "donor_whale")]
    DonorWhale,
    #[display(fmt = "Mega Donor")]
    #[serde(alias = "donor_mega")]
    DonorMega,
}

#[derive(Clone, Debug, Default, PartialEq, Display)]
pub enum XvbNode {
    #[display(fmt = "XvB North America Node")]
    NorthAmerica,
    #[default]
    #[display(fmt = "XvB European Node")]
    Europe,
    #[display(fmt = "Local P2pool")]
    P2pool,
}
impl XvbNode {
    pub fn url(&self) -> String {
        match self {
            Self::NorthAmerica => String::from(XVB_NODE_NA),
            Self::Europe => String::from(XVB_NODE_EU),
            Self::P2pool => String::from("127.0.0.1"),
        }
    }
    pub fn port(&self) -> String {
        match self {
            Self::NorthAmerica | Self::Europe => String::from(XVB_NODE_PORT),
            Self::P2pool => String::from("3333"),
        }
    }
    pub fn user(&self, address: &str) -> String {
        match self {
            Self::NorthAmerica => address.chars().take(8).collect(),
            Self::Europe => address.chars().take(8).collect(),
            Self::P2pool => GUPAX_VERSION_UNDERSCORE.to_string(),
        }
    }
    pub fn tls(&self) -> bool {
        match self {
            Self::NorthAmerica => true,
            Self::Europe => true,
            Self::P2pool => false,
        }
    }
    pub fn keepalive(&self) -> bool {
        match self {
            Self::NorthAmerica => true,
            Self::Europe => true,
            Self::P2pool => false,
        }
    }

    pub async fn update_fastest_node(
        client: &Arc<Client<HttpConnector>>,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
        gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
        process_xvb: &Arc<Mutex<Process>>,
    ) {
        let client_eu = client.clone();
        let client_na = client.clone();
        let ms_eu = spawn(async move { XvbNode::ping(&XvbNode::Europe.url(), &client_eu).await });
        let ms_na =
            spawn(async move { XvbNode::ping(&XvbNode::NorthAmerica.url(), &client_na).await });
        let node = if let Ok(ms_eu) = ms_eu.await {
            if let Ok(ms_na) = ms_na.await {
                // if two nodes are up, compare ping latency and return fastest.
                if ms_na != TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    if ms_na < ms_eu {
                        XvbNode::NorthAmerica
                    } else {
                        XvbNode::Europe
                    }
                } else
                // if only na is online, return it.
                if ms_na != TIMEOUT_NODE_PING && ms_eu == TIMEOUT_NODE_PING {
                    XvbNode::NorthAmerica
                } else
                // if only eu is online, return it.
                if ms_na == TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    XvbNode::Europe
                } else {
                    // if P2pool is returned, it means none of the two nodes are available.
                    XvbNode::P2pool
                }
            } else {
                error!("ping has failed !");
                XvbNode::P2pool
            }
        } else {
            error!("ping has failed !");
            XvbNode::P2pool
        };
        if node == XvbNode::P2pool {
            // if both nodes are dead, then the state of the process must be NodesOffline
            info!("XvB node ping, all offline or ping failed, switching back to local p2pool",);
            output_console(
                &gui_api_xvb,
                "XvB node ping, all offline or ping failed, switching back to local p2pool",
            );
            lock!(process_xvb).state = ProcessState::OfflineNodesAll;
        } else {
            // if node is up and because update_fastest is used only if token/address is valid, it means XvB process is Alive.
            info!("XvB node ping, both online and best is {}", node.url());
            output_console(
                &gui_api_xvb,
                &format!("XvB node ping, {} is selected as the fastest.", node),
            );
            lock!(process_xvb).state = ProcessState::Alive;
        }
        lock!(pub_api_xvb).stats_priv.node = node;
    }
    async fn ping(ip: &str, client: &Client<HttpConnector>) -> u128 {
        let request = Request::builder()
            .method("POST")
            .uri("http://".to_string() + ip + ":" + XVB_NODE_RPC + "/json_rpc")
            .body(hyper::Body::from(
                r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#,
            ))
            .expect("hyper request should build.");
        let ms;
        let now = Instant::now();
        match tokio::time::timeout(Duration::from_secs(5), client.request(request)).await {
            Ok(Ok(json_rpc)) => {
                // Attempt to convert to JSON-RPC.
                match hyper::body::to_bytes(json_rpc.into_body()).await {
                    Ok(b) => match serde_json::from_slice::<GetInfo<'_>>(&b) {
                        Ok(rpc) => {
                            if rpc.result.mainnet && rpc.result.synchronized {
                                ms = now.elapsed().as_millis();
                            } else {
                                ms = TIMEOUT_NODE_PING;
                                warn!("Ping | {ip} responded with valid get_info but is not in sync, remove this node!");
                            }
                        }
                        _ => {
                            ms = TIMEOUT_NODE_PING;
                            warn!("Ping | {ip} responded but with invalid get_info, remove this node!");
                        }
                    },
                    _ => ms = TIMEOUT_NODE_PING,
                };
            }
            _ => ms = TIMEOUT_NODE_PING,
        };
        ms
    }
}
impl PubXvbApi {
    pub fn new() -> Self {
        Self::default()
    }
    // The issue with just doing [gui_api = pub_api] is that values get overwritten.
    // This doesn't matter for any of the values EXCEPT for the output,  so we must
    // manually append it instead of overwriting.
    // This is used in the "helper" thread.
    pub(super) fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        let mut output = std::mem::take(&mut gui_api.output);
        let buf = std::mem::take(&mut pub_api.output);
        if !buf.is_empty() {
            output.push_str(&buf);
        }
        let tick = std::mem::take(&mut gui_api.tick);
        let tick_distribute_hr = std::mem::take(&mut gui_api.tick_distribute_hr);
        *gui_api = Self {
            output,
            tick,
            tick_distribute_hr,
            ..pub_api.clone()
        };
    }
}

fn signal_interrupt(
    process: Arc<Mutex<Process>>,
    start: Instant,
    client_http: &Arc<Client<HttpConnector>>,
    pub_api: &Arc<Mutex<PubXvbApi>>,
    gui_api: &Arc<Mutex<PubXvbApi>>,
    gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
    state_p2pool: &crate::disk::state::P2pool,
    state_xmrig: &crate::disk::state::Xmrig,
) -> bool {
    // Check SIGNAL
    // check if STOP or RESTART Signal is given.
    // if STOP, will put Signal to None, if Restart to Wait
    // in either case, will break from loop.
    if lock!(process).signal == ProcessSignal::Stop {
        debug!("P2Pool Watchdog | Stop SIGNAL caught");
        // Wait to get the exit status
        let uptime = start.elapsed();
        info!(
            "Xvb Watchdog | Stopped ... Uptime was: [{}]",
            Uptime::from(uptime)
        );
        // insert the signal into output of XvB
        // This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
        output_console(
            &gui_api,
            &format!("{}XvB stopped\n{}\n", HORI_CONSOLE, HORI_CONSOLE),
        );
        debug!("XvB Watchdog | Stop SIGNAL done, breaking");
        lock!(process).signal = ProcessSignal::None;
        lock!(process).state = ProcessState::Dead;
        return true;
    // Check RESTART
    } else if lock!(process).signal == ProcessSignal::Restart {
        debug!("XvB Watchdog | Restart SIGNAL caught");
        let uptime = Uptime::from(start.elapsed());
        info!("XvB Watchdog | Stopped ... Uptime was: [{}]", uptime);
        // no output to console because service will be started with fresh output.
        debug!("XvB Watchdog | Restart SIGNAL done, breaking");
        lock!(process).state = ProcessState::Waiting;
        return true;
    // Check UPDATE NODES
    } else if lock!(process).signal == ProcessSignal::UpdateNodes
        && lock!(process).state != ProcessState::Waiting
    {
        info!("XvB Watchdog | Signal has been given to ping and reselect Nodes.");
        // if signal is waiting, he is restarting or already updating nodes.
        // A signal has been given to ping the nodes and select the fastest.
        let gui_api_c = gui_api.clone();
        let pub_api_c = pub_api.clone();
        let gui_api_xmrig_c = gui_api_xmrig.clone();
        let client_http_c = client_http.clone();
        let process_c = process.clone();
        let token_xmrig = state_xmrig.token.clone();
        let address = state_p2pool.address.clone();
        lock!(process).state = ProcessState::Waiting;
        let node = lock!(gui_api).stats_priv.node.clone();
        spawn(async move {
            XvbNode::update_fastest_node(&client_http_c, &gui_api_c, &pub_api_c, &process_c).await;

            if let Err(err) = PrivXmrigApi::update_xmrig_config(
                &client_http_c,
                XMRIG_CONFIG_URI,
                &token_xmrig,
                &node,
                &address,
                &gui_api_xmrig_c,
            )
            .await
            {
                // show to console error about updating xmrig config
                output_console(
                    &gui_api_c,
                    &format!(
                        "Failure to update xmrig config with HTTP API.\nError: {}",
                        err
                    ),
                );
            } else {
                output_console(
                    &gui_api_c,
                    &format!("XvB node failed, falling back to {}", node),
                );
            }
        });
        lock!(process).signal = ProcessSignal::None;
        // the state will be Offline or Alive after update_fastest_node is done, meanwhile Signal will be None so not re-treated before update_fastest is done.
    }
    false
}

// print date time to console output in same format than xmrig
use chrono::Local;
fn datetime_console() -> String {
    format!("[{}]  ", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
}
pub fn output_console(gui_api: &Arc<Mutex<PubXvbApi>>, msg: &str) {
    if let Err(e) = writeln!(lock!(gui_api).output, "{}{msg}", datetime_console()) {
        error!("XvB Watchdog | GUI status write failed: {}", e);
    }
}
//---------------------------------------------------------------------------------------------------- TEST
#[cfg(test)]
mod test {
    // use std::{sync::Arc, thread, time::Instant};

    // use crate::{
    //     app::App,
    //     disk::state::XvbNode,
    //     helper::{xmrig::PrivXmrigApi, Helper},
    //     utils::constants::XMRIG_CONFIG_URI,
    // };

    use std::thread;

    use super::XvbPubStats;
    use hyper::Client;
    use hyper_tls::HttpsConnector;

    #[test]
    fn public_api_deserialize() {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build(https);
        let new_data = thread::spawn(move || corr(client)).join().unwrap();
        assert!(!new_data.reward_yearly.is_empty());
    }
    #[tokio::main]
    async fn corr(client: Client<HttpsConnector<hyper::client::HttpConnector>>) -> XvbPubStats {
        XvbPubStats::request_api(&client).await.unwrap()
    }
    // #[test]
    // fn update_xmrig_config() {
    //     let client: hyper::Client<hyper::client::HttpConnector> =
    //         hyper::Client::builder().build(hyper::client::HttpConnector::new());
    //     let node = XvbNode::Europe;
    //     let api_uri = ["http://127.0.0.1:18088/", XMRIG_CONFIG_URI].concat();
    //     // start app
    //     let app = App::new(Instant::now());
    //     // start xmrig

    //     Helper::start_xmrig(
    //         &app.helper,
    //         &app.state.xmrig,
    //         &app.state.gupax.absolute_xmrig_path,
    //         Arc::clone(&app.sudo),
    //     );
    //     let token = app.state.xmrig.token;
    //     let address = app.state.p2pool.address;
    //     // change config
    //     thread::spawn(move || req_update_config(client, &api_uri, &token, node, &address))
    //         .join()
    //         .unwrap();
    // }
    // #[tokio::main]
    // async fn req_update_config(
    //     client: hyper::Client<hyper::client::HttpConnector>,
    //     api_uri: &str,
    //     token: &str,
    //     node: XvbNode,
    //     address: &str,
    // ) {
    //     PrivXmrigApi::replace_xmrig_config(client, &api_uri, token, node, address)
    //         .await
    //         .unwrap()
    // }
}
