// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022-2023 hinto-janai
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

use crate::components::update::get_user_agent;
use crate::{constants::*, macros::*};
use egui::Color32;
use log::*;
use rand::{Rng, thread_rng};
use reqwest::{Client, RequestBuilder};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

//---------------------------------------------------------------------------------------------------- Node list
// Remote Monero Nodes with ZMQ enabled.
// The format is an array of tuples consisting of: (IP, LOCATION, RPC_PORT, ZMQ_PORT)

pub const REMOTE_NODES: [(&str, &str, &str, &str); 9] = [
    ("monero.10z.com.ar", "Argentina", "18089", "18084"),
    ("node.monerodevs.org", "Canada", "18089", "18084"),
    ("p2pmd.xmrvsbeast.com", "Germany", "18081", "18083"),
    ("node2.monerodevs.org", "France", "18089", "18084"),
    ("p2pool.uk", "United Kingdom", "18089", "18084"),
    ("xmr.support", "United States", "18081", "18083"),
    ("xmrbandwagon.hopto.org", "United States", "18081", "18084"),
    ("xmr.spotlightsound.com", "United States", "18081", "18084"),
    ("node.richfowler.net", "United States", "18089", "18084"),
];

pub const REMOTE_NODE_LENGTH: usize = REMOTE_NODES.len();

#[allow(dead_code)]
pub struct RemoteNode {
    pub ip: &'static str,
    pub location: &'static str,
    pub rpc: &'static str,
    pub zmq: &'static str,
}

impl Default for RemoteNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteNode {
    pub fn new() -> Self {
        Self::get_random_same_ok()
    }

    pub fn check_exists(og_ip: &str) -> String {
        for (ip, _, _, _) in REMOTE_NODES {
            if og_ip == ip {
                info!("Found remote node in array: {}", ip);
                return ip.to_string();
            }
        }
        let ip = REMOTE_NODES[0].0.to_string();
        warn!(
            "[{}] remote node does not exist, returning default: {}",
            og_ip, ip
        );
        ip
    }

    // Returns a default if index is not found in the const array.
    pub fn from_index(index: usize) -> Self {
        if index > REMOTE_NODE_LENGTH {
            Self::new()
        } else {
            let (ip, location, rpc, zmq) = REMOTE_NODES[index];
            Self {
                ip,
                location,
                rpc,
                zmq,
            }
        }
    }

    pub fn get_ip_rpc_zmq(og_ip: &str) -> (&str, &str, &str) {
        for (ip, _, rpc, zmq) in REMOTE_NODES {
            if og_ip == ip {
                return (ip, rpc, zmq);
            }
        }
        let (ip, _, rpc, zmq) = REMOTE_NODES[0];
        (ip, rpc, zmq)
    }

    // Return a random node (that isn't the one already selected).
    pub fn get_random(current_ip: &str) -> String {
        let mut rng = thread_rng().gen_range(0..REMOTE_NODE_LENGTH);
        let mut node = REMOTE_NODES[rng].0;
        while current_ip == node {
            rng = thread_rng().gen_range(0..REMOTE_NODE_LENGTH);
            node = REMOTE_NODES[rng].0;
        }
        node.to_string()
    }

    // Return a random valid node (no input str).
    pub fn get_random_same_ok() -> Self {
        let rng = thread_rng().gen_range(0..REMOTE_NODE_LENGTH);
        Self::from_index(rng)
    }

    // Return the node [-1] of this one
    pub fn get_last(current_ip: &str) -> String {
        let mut found = false;
        let mut last = current_ip;
        for (ip, _, _, _) in REMOTE_NODES {
            if found {
                return ip.to_string();
            }
            if current_ip == ip {
                found = true;
            } else {
                last = ip;
            }
        }
        last.to_string()
    }

    // Return the node [+1] of this one
    pub fn get_next(current_ip: &str) -> String {
        let mut found = false;
        for (ip, _, _, _) in REMOTE_NODES {
            if found {
                return ip.to_string();
            }
            if current_ip == ip {
                found = true;
            }
        }
        current_ip.to_string()
    }

    // This returns relative to the ping.
    pub fn get_last_from_ping(current_ip: &str, nodes: &Vec<NodeData>) -> String {
        let mut found = false;
        let mut last = current_ip;
        for data in nodes {
            if found {
                return last.to_string();
            }
            if current_ip == data.ip {
                found = true;
            } else {
                last = data.ip;
            }
        }
        last.to_string()
    }

    pub fn get_next_from_ping(current_ip: &str, nodes: &Vec<NodeData>) -> String {
        let mut found = false;
        for data in nodes {
            if found {
                return data.ip.to_string();
            }
            if current_ip == data.ip {
                found = true;
            }
        }
        current_ip.to_string()
    }
}

impl std::fmt::Display for RemoteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self.ip)
    }
}

//---------------------------------------------------------------------------------------------------- Formatting
// 5000 = 4 max length
pub fn format_ms(ms: u128) -> String {
    match ms.to_string().len() {
        1 => format!("{ms}ms   "),
        2 => format!("{ms}ms  "),
        3 => format!("{ms}ms "),
        _ => format!("{ms}ms"),
    }
}

// format_ip_location(monero1.heitechsoft.com) -> "monero1.heitechsoft.com | XX - LOCATION"
// [extra_space] controls whether extra space is appended so the list aligns.
pub fn format_ip_location(og_ip: &str, extra_space: bool) -> String {
    for (ip, location, _, _) in REMOTE_NODES {
        if og_ip == ip {
            let ip = if extra_space {
                format_ip(ip)
            } else {
                ip.to_string()
            };
            return format!("{ip} | {location}");
        }
    }
    "??? | ???".to_string()
}

pub fn format_ip(ip: &str) -> String {
    format!("{ip: >22}")
}

//---------------------------------------------------------------------------------------------------- Node data
pub const GREEN_NODE_PING: u128 = 100;
// yellow is anything in-between green/red
pub const RED_NODE_PING: u128 = 300;
pub const TIMEOUT_NODE_PING: u128 = 1000;

#[derive(Debug, Clone)]
pub struct NodeData {
    pub ip: &'static str,
    pub ms: u128,
    pub color: Color32,
}

impl NodeData {
    pub fn new_vec() -> Vec<Self> {
        let mut vec = Vec::new();
        for (ip, _, _, _) in REMOTE_NODES {
            vec.push(Self {
                ip,
                ms: 0,
                color: Color32::LIGHT_GRAY,
            });
        }
        vec
    }
}

//---------------------------------------------------------------------------------------------------- `/get_info`
// A struct repr of the JSON-RPC we're
// expecting back from the pinged nodes.
//
// This struct leaves out most fields on purpose,
// we only need a few to verify the node is ok.
#[allow(dead_code)] // allow dead code because Deserialize doesn't use all the fields in this program
#[derive(Debug, serde::Deserialize)]
pub struct GetInfo<'a> {
    pub id: &'a str,
    pub jsonrpc: &'a str,
    pub result: GetInfoResult,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetInfoResult {
    pub mainnet: bool,
    pub synchronized: bool,
}

//---------------------------------------------------------------------------------------------------- Ping data
#[derive(Debug)]
pub struct Ping {
    pub nodes: Vec<NodeData>,
    pub fastest: &'static str,
    pub pinging: bool,
    pub msg: String,
    pub prog: f32,
    pub pinged: bool,
    pub auto_selected: bool,
}

impl Default for Ping {
    fn default() -> Self {
        Self::new()
    }
}

impl Ping {
    pub fn new() -> Self {
        Self {
            nodes: NodeData::new_vec(),
            fastest: REMOTE_NODES[0].0,
            pinging: false,
            msg: "No ping in progress".to_string(),
            prog: 0.0,
            pinged: false,
            auto_selected: true,
        }
    }

    //---------------------------------------------------------------------------------------------------- Main Ping function
    #[cold]
    #[inline(never)]
    // Intermediate function for spawning thread
    pub fn spawn_thread(ping: &Arc<Mutex<Self>>) {
        info!("Spawning ping thread...");
        let ping = Arc::clone(ping);
        std::thread::spawn(move || {
            let now = Instant::now();
            match Self::ping(&ping) {
                Ok(msg) => {
                    info!("Ping ... OK");
                    ping.lock().unwrap().msg = msg;
                    ping.lock().unwrap().pinged = true;
                    ping.lock().unwrap().auto_selected = false;
                    ping.lock().unwrap().prog = 100.0;
                }
                Err(err) => {
                    error!("Ping ... FAIL ... {}", err);
                    ping.lock().unwrap().pinged = false;
                    ping.lock().unwrap().msg = err.to_string();
                }
            }
            info!("Ping ... Took [{}] seconds...", now.elapsed().as_secs_f32());
            ping.lock().unwrap().pinging = false;
        });
    }

    // This is for pinging the remote nodes to
    // find the fastest/slowest one for the user.
    // The process:
    //   - Send [get_info] JSON-RPC request over HTTP to all IPs
    //   - Measure each request in milliseconds
    //   - Timeout on requests over 5 seconds
    //   - Add data to appropriate struct
    //   - Sorting fastest to lowest is automatic (fastest nodes return ... the fastest)
    //
    // This used to be done 3x linearly but after testing, sending a single
    // JSON-RPC call to all IPs asynchronously resulted in the same data.
    //
    // <300ms  = GREEN
    // >300ms = YELLOW
    // >500ms = RED
    // timeout = RED
    // default = GRAY
    #[cold]
    #[inline(never)]
    #[tokio::main]
    pub async fn ping(ping: &Arc<Mutex<Self>>) -> Result<String, anyhow::Error> {
        // Start ping
        let ping = Arc::clone(ping);
        ping.lock().unwrap().pinging = true;
        ping.lock().unwrap().prog = 0.0;
        let percent = (100.0 / (REMOTE_NODE_LENGTH as f32)).floor();

        // Create HTTP client
        let info = "Creating HTTP Client".to_string();
        ping.lock().unwrap().msg = info;
        let client = Client::new();

        // Random User Agent
        let rand_user_agent = get_user_agent();
        // Handle vector
        let mut handles = Vec::with_capacity(REMOTE_NODE_LENGTH);
        let node_vec = arc_mut!(Vec::with_capacity(REMOTE_NODE_LENGTH));

        for (ip, _, rpc, _zmq) in REMOTE_NODES {
            let client = client.clone();
            let ping = Arc::clone(&ping);
            let node_vec = Arc::clone(&node_vec);
            let request = client
                .post("http://".to_string() + ip + ":" + rpc + "/json_rpc")
                .header("User-Agent", rand_user_agent)
                .body(r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#);

            let handle = tokio::task::spawn(async move {
                Self::response(request, ip, ping, percent, node_vec).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }

        let mut node_vec = std::mem::take(&mut *node_vec.lock().unwrap());
        node_vec.sort_by(|a, b| a.ms.cmp(&b.ms));
        let fastest_info = format!("Fastest node: {}ms ... {}", node_vec[0].ms, node_vec[0].ip);

        let info = "Cleaning up connections".to_string();
        info!("Ping | {}...", info);
        let mut ping = ping.lock().unwrap();
        ping.fastest = node_vec[0].ip;
        ping.nodes = node_vec;
        ping.msg = info;
        drop(ping);
        Ok(fastest_info)
    }

    #[cold]
    #[inline(never)]
    async fn response(
        request: RequestBuilder,
        ip: &'static str,
        ping: Arc<Mutex<Self>>,
        percent: f32,
        node_vec: Arc<Mutex<Vec<NodeData>>>,
    ) {
        // test multiples request as first can apparently timeout.
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

        let info = format!("{ms}ms ... {ip}");
        info!("Ping | {ms}ms ... {ip}");
        info!("{:?}", vec_ms);

        let color = if ms < GREEN_NODE_PING {
            GREEN
        } else if ms < RED_NODE_PING {
            YELLOW
        } else {
            RED
        };

        let mut ping = ping.lock().unwrap();
        ping.msg = info;
        ping.prog += percent;
        drop(ping);
        node_vec.lock().unwrap().push(NodeData { ip, ms, color });
    }
}
//---------------------------------------------------------------------------------------------------- NODE
#[cfg(test)]
mod test {
    use log::error;
    use reqwest::Client;

    use crate::components::node::{REMOTE_NODE_LENGTH, REMOTE_NODES, format_ip};
    use crate::components::update::get_user_agent;
    // Iterate through all nodes, find the longest domain.
    pub const REMOTE_NODE_MAX_CHARS: usize = {
        let mut len = 0;
        let mut index = 0;

        while index < REMOTE_NODE_LENGTH {
            let (node, _, _, _) = REMOTE_NODES[index];
            if node.len() > len {
                len = node.len();
            }
            index += 1;
        }

        assert!(len != 0);
        len
    };
    #[test]
    fn validate_node_ips() {
        for (ip, location, rpc, zmq) in REMOTE_NODES {
            assert!(ip.len() < 255);
            assert!(ip.is_ascii());
            assert!(!location.is_empty());
            assert!(!ip.is_empty());
            assert!(rpc == "18081" || rpc == "18089");
            assert!(zmq == "18083" || zmq == "18084");
        }
    }

    #[test]
    fn spacing() {
        for (ip, _, _, _) in REMOTE_NODES {
            assert!(format_ip(ip).len() <= REMOTE_NODE_MAX_CHARS);
        }
    }

    // This one pings the IPs defined in [REMOTE_NODES] and fully serializes the JSON data to make sure they work.
    // This will only be ran with be ran with [cargo test -- --ignored].
    #[tokio::test]
    #[ignore]
    async fn full_ping() {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize, Serialize)]
        struct GetInfo {
            id: String,
            jsonrpc: String,
        }

        // Create HTTP client
        let client = Client::new();

        // Random User Agent
        let rand_user_agent = get_user_agent();

        // Only fail this test if >50% of nodes fail.
        const HALF_REMOTE_NODES: usize = REMOTE_NODE_LENGTH / 2;
        // A string buffer to append the failed node data.
        let mut failures = String::new();
        let mut failure_count = 0;

        let mut n = 1;
        'outer: for (ip, _, rpc, zmq) in REMOTE_NODES {
            println!("[{n}/{REMOTE_NODE_LENGTH}] {ip} | {rpc} | {zmq}");
            let client = client.clone();
            // Try 3 times before failure
            let mut i = 1;
            let response = loop {
                let request = client
                    .post("http://".to_string() + ip + ":" + rpc + "/json_rpc")
                    .header("User-Agent", rand_user_agent)
                    .body(r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#);

                match request.send().await {
                    Ok(response) => break response,
                    Err(e) => {
                        println!("{:#?}", e);
                        if i >= 3 {
                            use std::fmt::Write;
                            let _ = writeln!(failures, "Node failure: {ip}:{rpc}:{zmq}");
                            failure_count += 1;
                            continue 'outer;
                        }
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        i += 1;
                    }
                }
            };
            let getinfo = response.json::<GetInfo>().await.unwrap();
            assert!(getinfo.id == "0");
            assert!(getinfo.jsonrpc == "2.0");
            n += 1;
        }

        let failure_percent = failure_count as f32 / HALF_REMOTE_NODES as f32;

        // If more than half the nodes fail, something
        // is definitely wrong, fail this test.
        if failure_count > HALF_REMOTE_NODES {
            error!("[{failure_percent:.2}% of nodes failed, failure log:\n{failures}");
        // If some failures happened, log.
        } else if failure_count != 0 {
            eprintln!("[{failure_count}] nodes failed ({failure_percent:.2}%):\n{failures}");
        } else {
            println!("No nodes failed - all OK");
        }
    }
}
