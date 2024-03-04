use derive_more::Display;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use log::{debug, info, warn};
use serde::Deserialize;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use crate::utils::{constants::XVB_URL_PUBLIC_API, macros::lock};

use super::Helper;

impl Helper {
    pub fn start_xvb(helper: &Arc<Mutex<Self>>) {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build(https);
        let gui_api = Arc::clone(&lock!(helper).gui_api_xvb);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xvb);
        thread::spawn(move || {
            Self::spawn_xvb_watchdog(client, gui_api, pub_api);
        });
    }
    #[tokio::main]
    async fn spawn_xvb_watchdog(
        client: hyper::Client<HttpsConnector<HttpConnector>>,
        gui_api: Arc<Mutex<PubXvbApi>>,
        pub_api: Arc<Mutex<PubXvbApi>>,
    ) {
        info!("XvB started");
        // Reset stats before loop
        *lock!(pub_api) = PubXvbApi::new();
        *lock!(gui_api) = PubXvbApi::new();

        info!("XvB | Entering watchdog mode... woof!");
        loop {
            // Set timer
            let now = Instant::now();
            debug!("XvB Watchdog | ----------- Start of loop -----------");
            // Send an HTTP API request
            debug!("XvB Watchdog | Attempting HTTP API request...");
            match PubXvbApi::request_xvb_public_api(client.clone(), XVB_URL_PUBLIC_API).await {
                Ok(new_data) => {
                    debug!("XvB Watchdog | HTTP API request OK");
                    let mut data = lock!(&pub_api);
                    *data = new_data;
                }
                Err(err) => {
                    warn!(
                        "XvB Watchdog | Could not send HTTP API request to: {}\n:{}",
                        XVB_URL_PUBLIC_API, err
                    );
                }
            }
            // XvB Status do not need to be refreshed like others because combine_with_gui do not refresh if no data is changed.
            let elapsed = now.elapsed().as_secs();
            if elapsed < 59 {
                let sleep = (60 - elapsed) as u64;
                debug!("XvB Watchdog | END OF LOOP - Sleeping for [{}]s...", sleep);
                std::thread::sleep(std::time::Duration::from_secs(sleep))
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
    #[serde(alias = "vip")]
    Vip,
    #[serde(alias = "donor")]
    Donor,
    #[serde(alias = "donor_vip")]
    DonorVip,
    #[serde(alias = "donor_whale")]
    DonorWhale,
    #[serde(alias = "donor_mega")]
    DonorMega,
}

impl PubXvbApi {
    pub fn new() -> Self {
        Self::default()
    }
    pub(super) fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        // update only if there is data, if no new data, pub_api fields are on default value.
        if !pub_api.reward_yearly.is_empty() {
            *gui_api = std::mem::take(pub_api)
        }
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
