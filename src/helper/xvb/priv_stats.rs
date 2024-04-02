use std::sync::{Arc, Mutex};

use anyhow::bail;
use log::{debug, error, warn};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::{
    helper::{xvb::output_console, Process, ProcessSignal, ProcessState},
    macros::lock,
    XVB_URL,
};

use super::{nodes::XvbNode, rounds::XvbRound, PubXvbApi};

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
    #[serde(skip)]
    // it is the time remaining before switching from P2pool to XvB or XvB to P2ool.
    // it is not the time remaining of the algo, even if it could be the same if never mining on XvB.
    pub time_switch_node: u32,
    #[serde(skip)]
    pub msg_indicator: String,
    #[serde(skip)]
    // so the hero mode can change between two decision of algorithm without restarting XvB.
    pub runtime_hero_mode: bool,
}

impl XvbPrivStats {
    pub async fn request_api(client: &Client, address: &str, token: &str) -> anyhow::Result<Self> {
        let resp = client
            .get(
                [
                    XVB_URL,
                    "/cgi-bin/p2pool_bonus_history_api.cgi?address=",
                    address,
                    "&token=",
                    token,
                ]
                .concat(),
            )
            .send()
            .await?;
        match resp.status() {
            StatusCode::OK => match resp.json::<Self>().await {
                Ok(s) => Ok(s),
                Err(err) => {
                    error!("XvB Watchdog | Data provided from private API is not deserializ-able.Error: {}", err);
                    bail!(
                        "Data provided from private API is not deserializ-able.Error: {}",
                        err
                    );
                }
            },
            StatusCode::UNPROCESSABLE_ENTITY => {
                bail!("the token is invalid for this xmr address.")
            }
            _ => bail!("The status of the response is not expected"),
        }
    }
    pub async fn update_stats(
        client: &Client,
        address: &str,
        token: &str,
        pub_api: &Arc<Mutex<PubXvbApi>>,
        gui_api: &Arc<Mutex<PubXvbApi>>,
        process: &Arc<Mutex<Process>>,
    ) {
        match XvbPrivStats::request_api(client, address, token).await {
            Ok(new_data) => {
                debug!("XvB Watchdog | HTTP API request OK");
                lock!(&pub_api).stats_priv = new_data;
            }
            Err(err) => {
                warn!(
                    "XvB Watchdog | Could not send HTTP private API request to: {}\n:{}",
                    XVB_URL, err
                );
                output_console(
                    gui_api,
                    &format!("Failure to retrieve private stats from {}", XVB_URL),
                );
                lock!(process).state = ProcessState::Failed;
                lock!(process).signal = ProcessSignal::Stop;
            }
        }
    }
}
