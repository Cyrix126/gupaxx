use std::sync::{Arc, Mutex};

use log::{debug, warn};
use reqwest::Client;
use serde::Deserialize;
use serde_this_or_that::as_u64;

use crate::{
    helper::{xvb::output_console, Process, ProcessSignal, ProcessState},
    macros::lock,
    XVB_URL_PUBLIC_API,
};

use super::{rounds::XvbRound, PubXvbApi};

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
    pub(in crate::helper) async fn request_api(
        client: &Client,
    ) -> std::result::Result<Self, anyhow::Error> {
        Ok(client
            .get(XVB_URL_PUBLIC_API)
            .send()
            .await?
            .json::<Self>()
            .await?)
    }
    pub async fn update_stats(
        client: &Client,
        gui_api: &Arc<Mutex<PubXvbApi>>,
        pub_api: &Arc<Mutex<PubXvbApi>>,
        process: &Arc<Mutex<Process>>,
    ) {
        debug!("XvB Watchdog | Attempting HTTP public API request...");
        match XvbPubStats::request_api(client).await {
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
                    gui_api,
                    &format!(
                        "Failure to retrieve public stats from {}",
                        XVB_URL_PUBLIC_API
                    ),
                );
                // we stop because we can't make the rest work without public stats. (winner in xvb private stats).
                lock!(process).state = ProcessState::Failed;
                lock!(process).signal = ProcessSignal::Stop;
            }
        }
    }
}
