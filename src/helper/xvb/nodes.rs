use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use derive_more::Display;
use log::{error, info, warn};
use reqwest_middleware::ClientWithMiddleware as Client;
use tokio::spawn;

use crate::{
    GUPAX_VERSION_UNDERSCORE, XVB_NODE_EU, XVB_NODE_NA, XVB_NODE_PORT, XVB_NODE_RPC,
    components::node::{GetInfo, TIMEOUT_NODE_PING},
    helper::{Process, ProcessName, ProcessState, xvb::output_console},
};

use super::PubXvbApi;
#[derive(Copy, Clone, Debug, Default, PartialEq, Display)]
pub enum XvbNode {
    #[display("XvB North America Node")]
    NorthAmerica,
    #[default]
    #[display("XvB European Node")]
    Europe,
    #[display("Local P2pool")]
    P2pool,
    #[display("Xmrig Proxy")]
    XmrigProxy,
}
impl XvbNode {
    pub fn url(&self) -> String {
        match self {
            Self::NorthAmerica => String::from(XVB_NODE_NA),
            Self::Europe => String::from(XVB_NODE_EU),
            Self::P2pool => String::from("127.0.0.1"),
            Self::XmrigProxy => String::from("127.0.0.1"),
        }
    }
    pub fn port(&self) -> String {
        match self {
            Self::NorthAmerica | Self::Europe => String::from(XVB_NODE_PORT),
            Self::P2pool => String::from("3333"),
            Self::XmrigProxy => String::from("3355"),
        }
    }
    pub fn user(&self, address: &str) -> String {
        match self {
            Self::NorthAmerica => address.chars().take(8).collect(),
            Self::Europe => address.chars().take(8).collect(),
            Self::P2pool => GUPAX_VERSION_UNDERSCORE.to_string(),
            Self::XmrigProxy => GUPAX_VERSION_UNDERSCORE.to_string(),
        }
    }
    pub fn tls(&self) -> bool {
        match self {
            Self::NorthAmerica => true,
            Self::Europe => true,
            Self::P2pool => false,
            Self::XmrigProxy => false,
        }
    }
    pub fn keepalive(&self) -> bool {
        match self {
            Self::NorthAmerica => true,
            Self::Europe => true,
            Self::P2pool => false,
            Self::XmrigProxy => false,
        }
    }

    pub async fn update_fastest_node(
        client: &Client,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
        gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
        process_xvb: &Arc<Mutex<Process>>,
    ) {
        let client_eu = client.clone();
        let client_na = client.clone();
        // two spawn to ping the two nodes in parallel and not one after the other.
        let ms_eu = spawn(async move {
            info!("Node | ping European XvB Node");
            XvbNode::ping(&XvbNode::Europe.url(), &client_eu).await
        });
        let ms_na = spawn(async move {
            info!("Node | ping North America Node");
            XvbNode::ping(&XvbNode::NorthAmerica.url(), &client_na).await
        });
        let node = if let Ok(ms_eu) = ms_eu.await {
            if let Ok(ms_na) = ms_na.await {
                // if two nodes are up, compare ping latency and return fastest.
                if ms_na != TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    if ms_na < ms_eu {
                        XvbNode::NorthAmerica
                    } else {
                        XvbNode::Europe
                    }
                } else if ms_na != TIMEOUT_NODE_PING && ms_eu == TIMEOUT_NODE_PING {
                    // if only na is online, return it.
                    XvbNode::NorthAmerica
                } else if ms_na == TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    // if only eu is online, return it.
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
                &mut gui_api_xvb.lock().unwrap().output,
                "XvB node ping, all offline or ping failed, switching back to local p2pool",
                ProcessName::Xvb,
            );
            process_xvb.lock().unwrap().state = ProcessState::OfflineNodesAll;
        } else {
            // if node is up and because update_fastest is used only if token/address is valid, it means XvB process is Alive.
            info!("XvB node ping, both online and best is {}", node.url());
            output_console(
                &mut gui_api_xvb.lock().unwrap().output,
                &format!("XvB node ping, {} is selected as the fastest.", node),
                ProcessName::Xvb,
            );
            info!("ProcessState to Syncing after finding joinable node");
            // could be used by xmrig who signal that a node is not joignable
            // or by the start of xvb
            // next iteration of the loop of XvB process will verify if all conditions are met to be alive.
            if process_xvb.lock().unwrap().state != ProcessState::Syncing {
                process_xvb.lock().unwrap().state = ProcessState::Syncing;
            }
        }
        pub_api_xvb.lock().unwrap().stats_priv.node = node;
    }
    async fn ping(ip: &str, client: &Client) -> u128 {
        let request = client
            .post("http://".to_string() + ip + ":" + XVB_NODE_RPC + "/json_rpc")
            .body(r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#);
        let mut vec_ms = vec![];
        for _ in 0..6 {
            // clone request
            let req = request
                .try_clone()
                .expect("should be able to clone a str body");
            // begin timer
            let now_req = Instant::now();
            // get and store time of request
            vec_ms.push(match tokio::time::timeout(Duration::from_millis(TIMEOUT_NODE_PING as u64), req.send()).await {
            Ok(Ok(json_rpc)) => {
                // Attempt to convert to JSON-RPC.
                match json_rpc.bytes().await {
                    Ok(b) => match serde_json::from_slice::<GetInfo<'_>>(&b) {
                        Ok(rpc) => {
                            if rpc.result.mainnet && rpc.result.synchronized {
                                now_req.elapsed().as_millis()
                            } else {
                                warn!("Ping | {ip} responded with valid get_info but is not in sync, remove this node!");
                                TIMEOUT_NODE_PING
                            }
                        }
                        _ => {
                            warn!("Ping | {ip} responded but with invalid get_info, remove this node!");
                            TIMEOUT_NODE_PING
                        }
                    },
                    _ => TIMEOUT_NODE_PING,
                }
            }
            _ => TIMEOUT_NODE_PING,
        });
        }
        let ms = *vec_ms
            .iter()
            .min()
            .expect("at least the value of timeout should be present");
        info!("Ping | {ms}ms ... {ip}");
        info!("{:?}", vec_ms);
        ms
    }
}
