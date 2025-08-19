// Gupaxx - Fork of Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex},
};

use derive_more::Display;
use log::{error, info};
use serde::Deserialize;
use tokio::spawn;

use crate::{
    GUPAX_VERSION_UNDERSCORE, XVB_NODE_EU, XVB_NODE_NA, XVB_NODE_PORT,
    components::node::TIMEOUT_NODE_PING,
    disk::state::{P2pool, Xvb},
    helper::{Process, ProcessName, ProcessState, p2pool::ImgP2pool, xvb::output_console},
    utils::node_latency::port_ping,
};

use super::PubXvbApi;
#[derive(Clone, Debug, Default, PartialEq, Display, Deserialize)]
pub enum Pool {
    #[display("XvB North America Pool")]
    XvBNorthAmerica,
    #[default]
    #[display("XvB European Pool")]
    XvBEurope,
    #[display("Local P2pool")]
    P2pool(u16),
    #[display("Xmrig Proxy")]
    XmrigProxy(u16),
    #[display("Custom Pool")]
    Custom(String, u16),
    #[display("Not connected to any pool")]
    Unknown,
}
impl Pool {
    pub fn url(&self) -> String {
        match self {
            Self::XvBNorthAmerica => String::from(XVB_NODE_NA),
            Self::XvBEurope => String::from(XVB_NODE_EU),
            Self::P2pool(_) => String::from("127.0.0.1"),
            Self::XmrigProxy(_) => String::from("127.0.0.1"),
            Self::Custom(url, _) => url.clone(),
            _ => "???".to_string(),
        }
    }
    pub fn port(&self) -> String {
        match self {
            Self::XvBNorthAmerica | Self::XvBEurope => String::from(XVB_NODE_PORT),
            Self::P2pool(port) => port.to_string(),
            Self::XmrigProxy(port) => port.to_string(),
            Self::Custom(_, port) => port.to_string(),
            _ => "???".to_string(),
        }
    }
    pub fn user(&self, address: &str) -> String {
        match self {
            Self::XvBNorthAmerica => address.chars().take(8).collect(),
            Self::XvBEurope => address.chars().take(8).collect(),
            _ => GUPAX_VERSION_UNDERSCORE.to_string(),
        }
    }
    pub fn tls(&self) -> bool {
        match self {
            Self::XvBNorthAmerica => true,
            Self::XvBEurope => true,
            Self::P2pool(_) => false,
            Self::XmrigProxy(_) => false,
            Self::Custom(_, _) => false,
            _ => false,
        }
    }
    pub fn keepalive(&self) -> bool {
        match self {
            Self::XvBNorthAmerica => true,
            Self::XvBEurope => true,
            Self::P2pool(_) => false,
            Self::XmrigProxy(_) => false,
            Self::Custom(_, _) => false,
            _ => false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_fastest_pool(
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
        gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
        process_xvb: &Arc<Mutex<Process>>,
        process_p2pool: &Arc<Mutex<Process>>,
        p2pool_img: &Arc<Mutex<ImgP2pool>>,
        p2pool_state: &P2pool,
        xvb_state: &Xvb,
    ) {
        if xvb_state.manual_pool_enabled {
            let manual_pool = if xvb_state.manual_pool_eu {
                Pool::XvBEurope
            } else {
                Pool::XvBNorthAmerica
            };
            info!("XvB node {} has been chosen manually", manual_pool.url());
            output_console(
                &mut gui_api_xvb.lock().unwrap().output,
                &format!("XvB node {manual_pool} has been chosen manually"),
                ProcessName::Xvb,
            );
            gui_api_xvb.lock().unwrap().stats_priv.pool = manual_pool;
            if process_xvb.lock().unwrap().state != ProcessState::Syncing {
                process_xvb.lock().unwrap().state = ProcessState::Syncing;
            }
            return;
        }
        // two spawn to ping the two nodes in parallel and not one after the other.
        let ms_eu_handle = spawn(async move {
            info!("Node | ping XvBEuropean XvB Node");
            let socket_address = format!("{}:{}", &Pool::XvBEurope.url(), &Pool::XvBEurope.port())
                .to_socket_addrs()
                .expect("hardcored valued should always convert to SocketAddr")
                .collect::<Vec<SocketAddr>>()[0];

            port_ping(socket_address, TIMEOUT_NODE_PING as u64).await
        });
        let ms_na_handle = spawn(async move {
            info!("Node | ping North America Node");
            let socket_address = format!(
                "{}:{}",
                &Pool::XvBNorthAmerica.url(),
                &Pool::XvBNorthAmerica.port()
            )
            .to_socket_addrs()
            .expect("hardcored valued should always convert to SocketAddr")
            .collect::<Vec<SocketAddr>>()[0];

            port_ping(socket_address, TIMEOUT_NODE_PING as u64).await
        });
        let ms_eu = ms_eu_handle
            .await
            .ok()
            .unwrap_or_else(|| Err(anyhow::Error::msg("")))
            .ok();
        let ms_na = ms_na_handle
            .await
            .ok()
            .unwrap_or_else(|| Err(anyhow::Error::msg("")))
            .ok();
        let pool = if let Some(ms_eu) = ms_eu {
            if let Some(ms_na) = ms_na {
                // if two nodes are up, compare ping latency and return fastest.
                if ms_na != TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    if ms_na < ms_eu {
                        Pool::XvBNorthAmerica
                    } else {
                        Pool::XvBEurope
                    }
                } else if ms_na != TIMEOUT_NODE_PING && ms_eu == TIMEOUT_NODE_PING {
                    // if only na is online, return it.
                    Pool::XvBNorthAmerica
                } else if ms_na == TIMEOUT_NODE_PING && ms_eu != TIMEOUT_NODE_PING {
                    // if only eu is online, return it.
                    Pool::XvBEurope
                } else {
                    // if P2pool is returned, it means none of the two nodes are available.
                    Pool::P2pool(p2pool_state.current_port(
                        process_p2pool.lock().unwrap().is_alive(),
                        &p2pool_img.lock().unwrap(),
                    ))
                }
            } else {
                error!("ping has failed !");
                Pool::P2pool(p2pool_state.current_port(
                    process_p2pool.lock().unwrap().is_alive(),
                    &p2pool_img.lock().unwrap(),
                ))
            }
        } else {
            error!("ping has failed !");
            Pool::P2pool(p2pool_state.current_port(
                process_p2pool.lock().unwrap().is_alive(),
                &p2pool_img.lock().unwrap(),
            ))
        };
        if pool
            == Pool::P2pool(p2pool_state.current_port(
                process_p2pool.lock().unwrap().is_alive(),
                &p2pool_img.lock().unwrap(),
            ))
        {
            // if both nodes are dead, then the state of the process must be NodesOffline
            info!("XvB node ping, all offline or ping failed, switching back to local p2pool",);
            output_console(
                &mut gui_api_xvb.lock().unwrap().output,
                "XvB node ping, all offline or ping failed, switching back to local p2pool",
                ProcessName::Xvb,
            );
            process_xvb.lock().unwrap().state = ProcessState::OfflinePoolsAll;
        } else {
            // if node is up and because update_fastest is used only if token/address is valid, it means XvB process is Alive.
            info!("XvB node ping, both online and best is {}", pool.url());
            output_console(
                &mut gui_api_xvb.lock().unwrap().output,
                &format!("XvB Pool ping, {pool} is selected as the fastest."),
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
        pub_api_xvb.lock().unwrap().stats_priv.pool = pool;
    }
}
