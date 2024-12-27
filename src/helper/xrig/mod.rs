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

use crate::helper::XvbNode;
use anyhow::Result;
use anyhow::anyhow;
use log::info;
use reqwest::header::AUTHORIZATION;
use reqwest_middleware::ClientWithMiddleware as Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub mod xmrig;
pub mod xmrig_proxy;

// update config of xmrig or xmrig-proxy
pub async fn update_xmrig_config(
    client: &Client,
    api_uri: &str,
    token: &str,
    node: &XvbNode,
    address: &str,
    rig: &str,
) -> Result<()> {
    // get config
    let request = client
        .get(api_uri)
        .header(AUTHORIZATION, ["Bearer ", token].concat());
    let mut config = request.send().await?.json::<Value>().await?;
    // modify node configuration
    let uri = [node.url(), ":".to_string(), node.port()].concat();
    info!(
        "replace xmrig from api url {api_uri} config with node {}",
        uri
    );
    *config
        .pointer_mut("/pools/0/url")
        .ok_or_else(|| anyhow!("pools/0/url does not exist in xmrig config"))? = uri.into();
    *config
        .pointer_mut("/pools/0/user")
        .ok_or_else(|| anyhow!("pools/0/user does not exist in xmrig config"))? = node
        .user(&address.chars().take(8).collect::<String>())
        .into();
    *config
        .pointer_mut("/pools/0/rig-id")
        .ok_or_else(|| anyhow!("pools/0/rig-id does not exist in xmrig config"))? = rig.into();
    *config
        .pointer_mut("/pools/0/tls")
        .ok_or_else(|| anyhow!("pools/0/tls does not exist in xmrig config"))? = node.tls().into();
    *config
        .pointer_mut("/pools/0/keepalive")
        .ok_or_else(|| anyhow!("pools/0/keepalive does not exist in xmrig config"))? =
        node.keepalive().into();
    // send new config
    client
        .put(api_uri)
        .header("Authorization", ["Bearer ", token].concat())
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(5))
        .body(config.to_string())
        .send()
        .await?;
    anyhow::Ok(())
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Hashrate {
    total: [Option<f32>; 3],
}
