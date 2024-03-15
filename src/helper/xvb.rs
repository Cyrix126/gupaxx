use anyhow::{bail, Result};
use bytes::Bytes;
use derive_more::Display;
use hyper::client::HttpConnector;
use hyper::StatusCode;
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};
use readable::up::Uptime;
use serde::Deserialize;
use std::fmt::Write;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::utils::constants::{XVB_PUBLIC_ONLY, XVB_URL};
use crate::{
    helper::{ProcessSignal, ProcessState},
    utils::{
        constants::{HORI_CONSOLE, XVB_URL_PUBLIC_API},
        macros::{lock, lock2, sleep},
    },
};

use super::p2pool::PubP2poolApi;
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
    ) {
        info!("XvB | Attempting to restart...");
        lock2!(helper, xvb).signal = ProcessSignal::Restart;
        lock2!(helper, xvb).state = ProcessState::Middle;
        let helper = helper.clone();
        let state_xvb = state_xvb.clone();
        let state_p2pool = state_p2pool.clone();
        // This thread lives to wait, start xmrig then die.
        thread::spawn(move || {
            while lock2!(helper, xvb).state != ProcessState::Waiting {
                warn!("XvB | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("XvB | Old process seems dead, starting new one!");
            Self::start_xvb(&helper, &state_xvb, &state_p2pool);
        });
        info!("XMRig | Restart ... OK");
    }
    pub fn start_xvb(
        helper: &Arc<Mutex<Self>>,
        state_xvb: &crate::disk::state::Xvb,
        state_p2pool: &crate::disk::state::P2pool,
    ) {
        lock2!(helper, xvb).state = ProcessState::Middle;

        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build(https);
        let gui_api = Arc::clone(&lock!(helper).gui_api_xvb);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xvb);
        let process = Arc::clone(&lock!(helper).xvb);
        // needed to see if it is alive. For XvB process to function completely, p2pool node must be alive to check the shares in the pplns window.
        let process_p2pool = Arc::clone(&lock!(helper).p2pool);
        let gui_api_p2pool = Arc::clone(&lock!(helper).gui_api_p2pool);
        let state_xvb_check = state_xvb.clone();
        let state_p2pool_check = state_p2pool.clone();
        // Reset before printing to output.
        // Need to reset because values of stats would stay otherwise which could bring confusion even if panel is with a disabled theme.
        *lock!(pub_api) = PubXvbApi::new();
        *lock!(gui_api) = PubXvbApi::new();
        // 2. Set process state
        debug!("XvB | Setting process state...");
        {
            let mut lock = lock!(process);
            lock.state = ProcessState::Middle;
            lock.signal = ProcessSignal::None;
            lock.start = Instant::now();
        }
        // verify if token and address are existent on XvB server
        let rt = tokio::runtime::Runtime::new().unwrap();
        let resp: anyhow::Result<()> = rt.block_on(async move {
            XvbPrivStats::request_api(&state_p2pool_check.address, &state_xvb_check.token).await?;
            Ok(())
        });
        match resp {
            Ok(_) => {
                let mut lock = lock!(process);
                lock.state = ProcessState::Alive;
            }
            Err(err) => {
                // send to console: token non existent for address on XvB server
                warn!("Xvb | Start ... Partially failed because token and associated address are not existent on XvB server: {}\n", err);
                // output the error to console
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "Failure to retrieve private stats from XvB server.\nError: {}\n{}\n",
                    err,
                    XVB_PUBLIC_ONLY
                ) {
                    error!("XvB Watchdog | GUI status write failed: {}", e);
                }
                lock!(process).state = ProcessState::NotMining;
            }
        }
        if !lock!(process_p2pool).is_alive() {
            // send to console: p2pool process is not running
            warn!("Xvb | Start ... Partially failed because P2pool instance is not running.");
            // output the error to console
            if let Err(e) = writeln!(
                lock!(gui_api).output,
                "\nFailure to completely start XvB process because p2pool instance is not running.\n",
            ) {
                error!("XvB Watchdog | GUI status write failed: {}", e);
            }

            lock!(process).state = ProcessState::Syncing;
        }
        if lock!(process).state != ProcessState::Alive {
            if let Err(e) = writeln!(lock!(gui_api).output, "\n{}\n", XVB_PUBLIC_ONLY,) {
                error!("XvB Watchdog | GUI status write failed: {}", e);
            }
        } else {
            info!("XvB started");
            if let Err(e) = writeln!(lock!(gui_api).output, "\nXvB started\n") {
                error!("XvB Watchdog | GUI status write failed: {}", e);
            }
        }
        let state_xvb_thread = state_xvb.clone();
        let state_p2pool_thread = state_p2pool.clone();
        thread::spawn(move || {
            Self::spawn_xvb_watchdog(
                client,
                gui_api,
                pub_api,
                process,
                &state_xvb_thread,
                &state_p2pool_thread,
                gui_api_p2pool,
                process_p2pool,
            );
        });
    }
    #[tokio::main]
    async fn spawn_xvb_watchdog(
        client: hyper::Client<HttpsConnector<HttpConnector>>,
        gui_api: Arc<Mutex<PubXvbApi>>,
        pub_api: Arc<Mutex<PubXvbApi>>,
        process: Arc<Mutex<Process>>,
        state_xvb: &crate::disk::state::Xvb,
        state_p2pool: &crate::disk::state::P2pool,
        gui_api_p2pool: Arc<Mutex<PubP2poolApi>>,
        process_p2pool: Arc<Mutex<Process>>,
    ) {
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
        info!("XvB | Entering watchdog mode... woof!");
        loop {
            debug!("XvB Watchdog | ----------- Start of loop -----------");
            // verify if p2pool node is running with correct token.
            if lock!(process).state != ProcessState::NotMining {
                if lock!(process_p2pool).is_alive() {
                    // verify if state is to changed
                    if lock!(process).state == ProcessState::Syncing {
                        info!("XvB | started this time with p2pool");
                        lock!(process).state = ProcessState::Alive;
                        if let Err(e) = writeln!(
                            lock!(gui_api).output,
                            "\nXvB is now started because p2pool node came online.\n",
                        ) {
                            error!("XvB Watchdog | GUI status write failed: {}", e);
                        }
                    }
                } else {
                    // verify if the state is changing because p2pool is not alive anymore.
                    if lock!(process).state != ProcessState::Syncing {
                        info!("XvB | stop partially because p2pool is not alive anymore.");
                        lock!(process).state = ProcessState::Alive;
                        if let Err(e) = writeln!(
                            lock!(gui_api).output,
                            "\nXvB is now partially stopped because p2pool node came offline.\n",
                        ) {
                            error!("XvB Watchdog | GUI status write failed: {}", e);
                        }
                        lock!(process).state = ProcessState::Syncing
                    }
                }
            }
            // Set timer
            let now = Instant::now();
            // check signal
            debug!("XvB | check signal");
            if signal_interrupt(process.clone(), start, gui_api.clone()) {
                break;
            }
            // verify if
            // Send an HTTP API request only if one minute is passed since the last.
            // if since is 0, send request because it's the first time.
            let since = lock!(gui_api).tick;
            if since >= 60 || since == 0 {
                debug!("XvB Watchdog | Attempting HTTP public API request...");
                match XvbPubStats::request_api(client.clone()).await {
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
                        if let Err(e) = writeln!(
                            lock!(gui_api).output,
                            "Failure to retrieve public stats from {}",
                            XVB_URL_PUBLIC_API
                        ) {
                            error!("XvB Watchdog | GUI status write failed: {}", e);
                        }
                        lock!(process).state = ProcessState::Failed;
                        break;
                    }
                }
                debug!("XvB Watchdog | Attempting HTTP private API request...");
                // only if private API is accessible, NotMining here means that the token and address is not registered on the XvB website.
                if lock!(process).state == ProcessState::Alive {
                    // reload private stats
                    match XvbPrivStats::request_api(&state_p2pool.address, &state_xvb.token).await {
                        Ok(b) => {
                            debug!("XvB Watchdog | HTTP API request OK");
                            let new_data = match serde_json::from_slice::<XvbPrivStats>(&b) {
                                Ok(data) => data,
                                Err(e) => {
                                    warn!("XvB Watchdog | Data provided from private API is not deserializ-able.Error: {}", e);
                                    // output the error to console
                                    if let Err(e) = writeln!(
                            lock!(gui_api).output,
                            "XvB Watchdog | Data provided from private API is not deserializ-able.Error: {}", e
                        ) {
                            error!("XvB Watchdog | GUI status write failed: {}", e);
                        }
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
                            // output the error to console
                            if let Err(e) = writeln!(
                                lock!(gui_api).output,
                                "Failure to retrieve private stats from {}",
                                XVB_URL
                            ) {
                                error!("XvB Watchdog | GUI status write failed: {}", e);
                            }
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
                            stats_priv.donor_1hr_avg / 1000.0,
                            stats_priv.donor_24hr_avg / 1000.0,
                        ) {
                            x if x.0 > 1000.0 && x.1 > 1000.0 => Some(XvbRound::DonorMega),
                            x if x.0 > 100.0 && x.1 > 100.0 => Some(XvbRound::DonorWhale),
                            x if x.0 > 10.0 && x.1 > 10.0 => Some(XvbRound::DonorVip),
                            x if x.0 > 1.0 && x.1 > 1.0 => Some(XvbRound::Donor),
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
}
//---------------------------------------------------------------------------------------------------- Public XvB API
use serde_this_or_that::as_u64;
#[derive(Debug, Clone, Default)]
pub struct PubXvbApi {
    pub output: String,
    pub uptime: u64,
    pub tick: u8,
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

impl XvbPubStats {
    #[inline]
    // Send an HTTP request to XvB's API, serialize it into [Self] and return it
    async fn request_api(
        client: hyper::Client<HttpsConnector<HttpConnector>>,
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
    pub async fn request_api(address: &str, token: &str) -> Result<Bytes> {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build(https);
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct XvbPrivStats {
    pub fails: u8,
    pub donor_1hr_avg: f32,
    pub donor_24hr_avg: f32,
    #[serde(skip)]
    pub win_current: bool,
    #[serde(skip)]
    pub round_participate: Option<XvbRound>,
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
        *gui_api = Self {
            output,
            tick,
            ..pub_api.clone()
        };
    }
}

fn signal_interrupt(
    process: Arc<Mutex<Process>>,
    start: Instant,
    gui_api: Arc<Mutex<PubXvbApi>>,
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
        if let Err(e) = writeln!(
            lock!(gui_api).output,
            "{}\nXvb stopped | Uptime: [{}] | \n{}\n\n\n\n",
            HORI_CONSOLE,
            Uptime::from(uptime),
            HORI_CONSOLE
        ) {
            error!("XvB Watchdog | GUI Uptime/Exit status write failed: {}", e);
        }
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
    }
    false
}
//---------------------------------------------------------------------------------------------------- TEST
#[cfg(test)]
mod test {
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
        dbg!(new_data);
    }
    #[tokio::main]
    async fn corr(client: Client<HttpsConnector<hyper::client::HttpConnector>>) -> XvbPubStats {
        XvbPubStats::request_api(client).await.unwrap()
    }
}
