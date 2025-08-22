use crate::helper::sleep;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::spawn,
    time::{Duration, Instant},
};

use enclose::enc;
use futures::StreamExt;
use log::info;
use monero_crawler_lib::{CrawlBuilder, capability_checkers::CapabilitiesChecker};
use tokio::time::sleep;

use crate::components::node::{RemoteNode, RemoteNodes};

pub struct Crawler {
    pub nodes: RemoteNodes,
    // crawling will keep running while it didn't find nb_nodes_fast number
    pub crawling: bool,
    pub stopping: bool,
    pub msg: String,
    pub prog: f32,
    pub requirements: CrawlerRequirements,
    pub rpc_ports: Vec<u16>,
    pub zmq_ports: Vec<u16>,
    pub timeout: Duration,
    pub handle: Option<Sender<bool>>,
}

impl Default for Crawler {
    fn default() -> Self {
        Crawler {
            nodes: RemoteNodes::default(),
            crawling: false,
            stopping: false,
            msg: "Inactive".to_string(),
            prog: 0.0,
            requirements: CrawlerRequirements::default(),
            rpc_ports: vec![18081, 18089],
            zmq_ports: vec![18083, 18084],
            timeout: Duration::new(10, 0),
            handle: None,
        }
    }
}

/// The crawler will keep running and replace the found nodes by faster ones until the number of fast nodes has been fulfilled or the time limit is reached.
pub struct CrawlerRequirements {
    // number of saved fast nodes after which the crawler will stop
    pub nb_nodes_fast: u8,
    // max ping after which the discovered node is not saved
    pub max_ping: u32,
    // maximum ping before the discovered node is considered not fast.
    // It can still be saved if it is under max_ping
    pub max_ping_fast: u32,
    // number of nodes that are not fast but will be saved anyway.
    // It allows the user to stop the crawl and use a medium fast node if the crawl did not find a fast node yet
    pub nb_nodes_medium: u8,
}

impl Default for CrawlerRequirements {
    fn default() -> Self {
        CrawlerRequirements {
            nb_nodes_fast: 1,
            max_ping: 300,
            max_ping_fast: 35,
            nb_nodes_medium: 3,
        }
    }
}

impl Crawler {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Crawler::default()))
    }
    pub fn start(crawler: &Arc<Mutex<Self>>) {
        info!("Spawning crawl thread...");

        let (tx, rx) = mpsc::channel();
        // do not start if the past crawling did not stopped yet
        while crawler.lock().unwrap().crawling {
            sleep!(20);
        }
        crawler.lock().unwrap().crawling = true;
        crawler.lock().unwrap().prog = 0.0;
        spawn(enc!((crawler) move || {
            let now = Instant::now();
            Self::crawl(&crawler, rx);
            info!(
                "Crawl... Took [{}] seconds to find the minimum required nodes",
                now.elapsed().as_secs_f32()
            );
        }));
        crawler.lock().unwrap().handle = Some(tx);
        // spawn a timeout

        spawn(enc!((crawler) move || {
            Self::update_progress(&crawler);
        }));
    }
    /// Used manually by the user
    /// Will do nothing if the crawling wasn't running
    pub fn stop(crawler: &Arc<Mutex<Self>>) {
        let crawler_lock = crawler.lock().unwrap();
        if crawler_lock.crawling
            && let Some(handle) = &crawler_lock.handle
        {
            handle.send(true).unwrap();
        }
    }

    #[tokio::main]
    pub async fn update_progress(crawler: &Arc<Mutex<Self>>) {
        let timeout = crawler.lock().unwrap().timeout;

        for i in 1..11 {
            sleep(timeout / 10).await;
            let mut crawler_lock = crawler.lock().unwrap();

            // always check that the crawling is still running
            if !crawler_lock.crawling {
                return;
            }

            // before adding 10% to progress because 10% of the timeout passed, check if the current progress is not already there because of fast node found.

            if crawler_lock.prog < i as f32 * 10.0 {
                crawler_lock.prog += 10.0;
            }
        }
        Self::stop(crawler);
    }

    #[tokio::main]
    pub async fn crawl(crawler: &Arc<Mutex<Self>>, terminate_rx: Receiver<bool>) {
        // reset the peers found
        crawler.lock().unwrap().nodes = RemoteNodes::default();
        let mut nb_nodes_fast = 0;
        let mut nb_nodes_medium = 0;
        let percent = 100.0 / (crawler.lock().unwrap().requirements.nb_nodes_fast as f32).floor();
        let crawl;
        {
            let crawler_lock = crawler.lock().unwrap();

            let max_ping = crawler_lock.requirements.max_ping;
            let zmq_ports = crawler_lock.zmq_ports.clone();
            let rpc_ports = crawler_lock.rpc_ports.clone();

            crawl = CrawlBuilder::default()
                .capabilities(vec![
                    CapabilitiesChecker::Latency(max_ping),
                    CapabilitiesChecker::Rpc(rpc_ports),
                    CapabilitiesChecker::Zmq(zmq_ports),
                    CapabilitiesChecker::SpyNode(false, vec![]),
                    CapabilitiesChecker::SeedNode(false),
                ])
                // .connections_limit(Arc::new(Semaphore::new(1)))
                .build()
                .unwrap();
        }

        // we want the crawler data to be accessible while the crawler is running
        let mut stream = crawl.discover_peers().await;
        while let Some((peer, rpc_port, zmq_port, ms)) = stream.next().await {
            let remote_node = RemoteNode {
                ip: peer.ip(),
                rpc: rpc_port,
                zmq: zmq_port,
                ms: ms as u64,
            };
            info!("Crawl | found a new compatible p2pool node !");
            let mut crawler_lock = crawler.lock().unwrap();

            match ms.cmp(&crawler_lock.requirements.max_ping_fast) {
                std::cmp::Ordering::Greater => {
                    if crawler_lock.requirements.nb_nodes_medium > 0 {
                        // if the max number of medium nodes is not reached, add the node to the list
                        if nb_nodes_medium < crawler_lock.requirements.nb_nodes_medium {
                            nb_nodes_medium += 1;
                            crawler_lock.nodes.push(remote_node);
                            crawler_lock.msg = format!(
                                "Discovered {} node{} with medium latency",
                                nb_nodes_medium,
                                // little hack for plurial, we will need to think about managing localization
                                if nb_nodes_medium > 1 { "s" } else { "" }
                            );
                        }
                        // if the max number of medium nodes is reached, replace the slowest one if the new one is faster.
                        else {
                            let index_slowest = crawler_lock.nodes.len() - 1;
                            if ms < crawler_lock.nodes[index_slowest].ms as u32 {
                                crawler_lock.nodes.remove(index_slowest);
                                crawler_lock.nodes.push(remote_node);
                                crawler_lock.msg =
                                    "Replaced the slowest node with a faster one".to_string();
                            }
                        }
                    }
                }
                _ => {
                    nb_nodes_fast += 1;
                    crawler_lock.nodes.push(remote_node);
                    // before adding progress, check if the progress is not already too far because of the timeout.
                    // Only add the progress if we are before the timeout progress.
                    if crawler_lock.prog < nb_nodes_fast as f32 * percent {
                        crawler_lock.prog += percent;
                    }
                    crawler_lock.msg = format!(
                        "Discovered {} node{} with fast latency",
                        nb_nodes_fast,
                        // little hack for plurial, we will need to think about managing localization
                        if nb_nodes_fast > 1 { "s" } else { "" }
                    );
                }
            }

            // sort by latency every time a new one is found
            crawler_lock.nodes.sort_by(|a, b| a.ms.cmp(&b.ms));

            // stop if the max number of fast nodes is reached
            if nb_nodes_fast == crawler_lock.requirements.nb_nodes_fast {
                crawler_lock.msg = "Discovered enough fast latency nodes".to_string();
                drop(crawler_lock);
                break;
            }

            // stop if a signal to terminate is received
            if terminate_rx.try_recv().is_ok() {
                if crawler_lock.prog < 100.0 {
                    crawler_lock.msg = "Stopped manually".to_string();
                } else {
                    crawler_lock.msg = "Stopped after reaching the timeout".to_string();
                }
                drop(crawler_lock);
                break;
            }
        }
        // since the crawling is stopping, we remove the handler that allows to stop it manually
        crawler.lock().unwrap().handle = None;
        // we only put the crawling to false once the crawling is really done, we don't want to have a second crawling happening when the old one is not yet done.
        crawler.lock().unwrap().crawling = false;
        crawler.lock().unwrap().stopping = false;
    }
}
