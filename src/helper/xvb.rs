use derive_more::Display;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::fmt::Write;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::{
    disk::state::Xvb,
    helper::{ProcessSignal, ProcessState},
    utils::{
        constants::{HORI_CONSOLE, XVB_URL_PUBLIC_API},
        human::HumanTime,
        macros::{lock, lock2, sleep},
    },
};

use super::{Helper, Process};

impl Helper {
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_xvb(helper: &Arc<Mutex<Self>>) {
        info!("XvB | Attempting to stop...");
        lock2!(helper, xvb).signal = ProcessSignal::Stop;
        lock2!(helper, xvb).state = ProcessState::Middle;
        // Reset stats
        let gui_api = Arc::clone(&lock!(helper).gui_api_xvb);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xvb);
        *lock!(pub_api) = PubXvbApi::new();
        *lock!(gui_api) = PubXvbApi::new();
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
        let state_xvb = state_xvb.clone();
        let state_p2pool = state_p2pool.clone();

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
            Xvb::is_token_exist(state_p2pool.address, state_xvb.token).await?;
            Ok(())
        });
        match resp {
            Ok(_) => {
                let mut lock = lock!(process);
                lock.state = ProcessState::Alive;
            }
            Err(err) => {
                // send to console: token non existent for address on XvB server
                warn!("Xvb | Start ... Failed because token and associated address are not existent on XvB server: {}", err);
                // output the error to console
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "Failure to retrieve private stats from XvB server.\nError: {}",
                    err
                ) {
                    error!("XvB Watchdog | GUI status write failed: {}", e);
                }
                lock2!(helper, xvb).state = ProcessState::NotMining;
            }
        }

        thread::spawn(move || {
            Self::spawn_xvb_watchdog(client, gui_api, pub_api, process);
        });
    }
    #[tokio::main]
    async fn spawn_xvb_watchdog(
        client: hyper::Client<HttpsConnector<HttpConnector>>,
        gui_api: Arc<Mutex<PubXvbApi>>,
        pub_api: Arc<Mutex<PubXvbApi>>,
        process: Arc<Mutex<Process>>,
    ) {
        info!("XvB started");

        if let Err(e) = writeln!(
            lock!(gui_api).output,
            "{}\nXvb started\n{}\n\n",
            HORI_CONSOLE,
            HORI_CONSOLE
        ) {
            error!("XvB Watchdog | GUI status write failed: {}", e);
        }
        let start = lock!(process).start;
        info!("XvB | Entering watchdog mode... woof!");
        loop {
            debug!("XvB Watchdog | ----------- Start of loop -----------");
            // Set timer
            let now = Instant::now();
            // check signal
            debug!("XvB | check signal");
            if signal_interrupt(process.clone(), start, gui_api.clone()) {
                break;
            }
            // Send an HTTP API request only if one minute is passed since the last.
            // if since is 0, send request because it's the first time.
            let since = lock!(gui_api).tick;
            if since >= 60 || since == 0 {
                // *lock!(pub_api) = PubXvbApi::new();
                // *lock!(gui_api) = PubXvbApi::new();
                debug!("XvB Watchdog | Attempting HTTP API request...");
                match PubXvbApi::request_xvb_public_api(client.clone(), XVB_URL_PUBLIC_API).await {
                    Ok(new_data) => {
                        debug!("XvB Watchdog | HTTP API request OK");
                        *lock!(&pub_api) = new_data;
                        lock!(gui_api).tick += 0;
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
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PubXvbApi {
    #[serde(skip)]
    pub output: String,
    #[serde(skip)]
    pub uptime: HumanTime,
    #[serde(skip)]
    pub tick: u8,
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
    #[inline]
    // Send an HTTP request to XvB's API, serialize it into [Self] and return it
    async fn request_xvb_public_api(
        client: hyper::Client<HttpsConnector<HttpConnector>>,
        api_uri: &str,
    ) -> std::result::Result<Self, anyhow::Error> {
        let request = hyper::Request::builder()
            .method("GET")
            .uri(api_uri)
            .body(hyper::Body::empty())?;
        let response =
            tokio::time::timeout(std::time::Duration::from_secs(8), client.request(request))
                .await?;
        // let response = client.request(request).await;

        let body = hyper::body::to_bytes(response?.body_mut()).await?;
        Ok(serde_json::from_slice::<Self>(&body)?)
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
        let uptime = HumanTime::into_human(start.elapsed());
        info!("Xvb Watchdog | Stopped ... Uptime was: [{}]", uptime);
        // insert the signal into output of XvB
        // This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
        if let Err(e) = writeln!(
            lock!(gui_api).output,
            "{}\nXvb stopped | Uptime: [{}] | \n{}\n\n\n\n",
            HORI_CONSOLE,
            uptime,
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
        let uptime = HumanTime::into_human(start.elapsed());
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

    use super::PubXvbApi;
    use crate::utils::constants::XVB_URL_PUBLIC_API;
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
    async fn corr(client: Client<HttpsConnector<hyper::client::HttpConnector>>) -> PubXvbApi {
        PubXvbApi::request_xvb_public_api(client, XVB_URL_PUBLIC_API)
            .await
            .unwrap()
    }
}
