use enclose::enc;
use log::{debug, error, info, warn};
use reqwest::{header::AUTHORIZATION, Client};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tokio::spawn;

use crate::{
    disk::state::Xmrig,
    helper::{
        check_died, check_user_input, signal_end, sleep_end_loop,
        xrig::update_xmrig_config,
        xvb::{nodes::XvbNode, PubXvbApi},
        Helper, Process, ProcessName, ProcessSignal, ProcessState,
    },
    macros::{arc_mut, lock, lock2, sleep},
    miscs::output_console,
    regex::{contains_timeout, contains_usepool, detect_new_node_xmrig, XMRIG_REGEX},
    GUPAX_VERSION_UNDERSCORE, UNKNOWN_DATA,
};
use crate::{XMRIG_CONFIG_URL, XMRIG_PROXY_SUMMARY_URL};

use super::xmrig::PubXmrigApi;
impl Helper {
    // Takes in some [State/XmrigProxy] and parses it to build the actual command arguments.
    // Returns the [Vec] of actual arguments,
    #[cold]
    #[inline(never)]
    pub async fn read_pty_xp(
        output_parse: Arc<Mutex<String>>,
        output_pub: Arc<Mutex<String>>,
        reader: Box<dyn std::io::Read + Send>,
        process_xvb: Arc<Mutex<Process>>,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
    ) {
        use std::io::BufRead;
        let mut stdout = std::io::BufReader::new(reader).lines();

        // Run a ANSI escape sequence filter for the first few lines.
        let mut i = 0;
        while let Some(Ok(line)) = stdout.next() {
            let line = strip_ansi_escapes::strip_str(line);
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("XMRig-Proxy PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("XMRig-Proxy PTY Pub | Output error: {}", e);
            }
            if i > 7 {
                break;
            } else {
                i += 1;
            }
        }

        while let Some(Ok(line)) = stdout.next() {
            // need to verify if node still working
            // for that need to catch "connect error"
            // only switch nodes of XvB if XvB process is used
            if lock!(process_xvb).is_alive() {
                if contains_timeout(&line) {
                    let current_node = lock!(pub_api_xvb).current_node;
                    if let Some(current_node) = current_node {
                        // updating current node to None, will stop sending signal of FailedNode until new node is set
                        // send signal to update node.
                        warn!(
                        "XMRig-Proxy PTY Parse | node is offline, sending signal to update nodes."
                    );
                        if current_node != XvbNode::P2pool {
                            lock!(process_xvb).signal = ProcessSignal::UpdateNodes(current_node);
                        }
                        lock!(pub_api_xvb).current_node = None;
                    }
                }
                if contains_usepool(&line) {
                    info!("XMRig-Proxy PTY Parse | new pool detected");
                    // need to update current node because it was updated.
                    // if custom node made by user, it is not supported because algo is deciding which node to use.

                    let node = detect_new_node_xmrig(&line);
                    if node.is_none() {
                        warn!(
                            "XMRig-Proxy PTY Parse | node is not understood, switching to backup."
                        );
                        // update with default will choose which XvB to prefer. Will update XvB to use p2pool.
                        lock!(process_xvb).signal = ProcessSignal::UpdateNodes(XvbNode::default());
                    }
                    lock!(pub_api_xvb).current_node = node;
                }
            }
            //			println!("{}", line); // For debugging.
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("XMRig-Proxy PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("XMRig-Proxy PTY Pub | Output error: {}", e);
            }
        }
    }
    pub fn build_xp_args(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::state::XmrigProxy,
    ) -> Vec<String> {
        let mut args = Vec::with_capacity(500);
        let api_ip;
        let api_port;
        let ip;
        let port;

        // [Simple]
        if state.simple {
            // Build the xmrig argument
            let rig = if state.simple_rig.is_empty() {
                GUPAX_VERSION_UNDERSCORE.to_string()
            } else {
                state.simple_rig.clone()
            }; // Rig name
            args.push("-o".to_string());
            args.push("127.0.0.1:3333".to_string()); // Local P2Pool (the default)
            args.push("-b".to_string());
            args.push("0.0.0.0:3355".to_string());
            args.push("--user".to_string());
            args.push(rig); // Rig name
            args.push("--no-color".to_string()); // No color
            args.push("--http-host".to_string());
            args.push("127.0.0.1".to_string()); // HTTP API IP
            args.push("--http-port".to_string());
            args.push("18089".to_string()); // HTTP API Port
            lock2!(helper, pub_api_xp).node = "127.0.0.1:3333 (Local P2Pool)".to_string();

        // [Advanced]
        } else {
            // XMRig doesn't understand [localhost]
            let p2pool_ip = if state.p2pool_ip == "localhost" || state.p2pool_ip.is_empty() {
                "127.0.0.1"
            } else {
                &state.p2pool_ip
            };
            api_ip = if state.api_ip == "localhost" || state.api_ip.is_empty() {
                "127.0.0.1".to_string()
            } else {
                state.api_ip.to_string()
            };
            api_port = if state.api_port.is_empty() {
                "18089".to_string()
            } else {
                state.api_port.to_string()
            };
            ip = if state.api_ip == "localhost" || state.ip.is_empty() {
                "0.0.0.0".to_string()
            } else {
                state.ip.to_string()
            };

            port = if state.port.is_empty() {
                "3355".to_string()
            } else {
                state.port.to_string()
            };
            let p2pool_url = format!("{}:{}", p2pool_ip, state.p2pool_port); // Combine IP:Port into one string
            let bind_url = format!("{}:{}", ip, port); // Combine IP:Port into one string
            args.push("--user".to_string());
            args.push(state.address.clone()); // Wallet
            args.push("--rig-id".to_string());
            args.push(state.rig.to_string()); // Rig ID
            args.push("-o".to_string());
            args.push(p2pool_url.clone()); // IP/Port
            args.push("-b".to_string());
            args.push(bind_url.clone()); // IP/Port
            args.push("--http-host".to_string());
            args.push(api_ip.to_string()); // HTTP API IP
            args.push("--http-port".to_string());
            args.push(api_port.to_string()); // HTTP API Port
            args.push("--no-color".to_string()); // No color escape codes
            if state.tls {
                args.push("--tls".to_string());
            } // TLS
            if state.keepalive {
                args.push("--keepalive".to_string());
            } // Keepalive
            lock2!(helper, pub_api_xp).node = p2pool_url;
        }
        args.push(format!("--http-access-token={}", state.token)); // HTTP API Port
        args.push("--http-no-restricted".to_string());
        args
    }

    pub fn stop_xp(helper: &Arc<Mutex<Self>>) {
        info!("XMRig-Proxy | Attempting to stop...");
        lock2!(helper, xmrig_proxy).signal = ProcessSignal::Stop;
        info!("locked signal ok");
        lock2!(helper, xmrig_proxy).state = ProcessState::Middle;
        info!("locked state ok");
        let gui_api = Arc::clone(&lock!(helper).gui_api_xp);
        info!("clone gui ok");
        let pub_api = Arc::clone(&lock!(helper).pub_api_xp);
        info!("clone pub ok");
        *lock!(pub_api) = PubXmrigProxyApi::new();
        info!("pub api reset ok");
        *lock!(gui_api) = PubXmrigProxyApi::new();
        info!("gui api reset ok");
    }
    // The "restart frontend" to a "frontend" function.
    // Basically calls to kill the current xmrig-proxy, waits a little, then starts the below function in a a new thread, then exit.
    pub fn restart_xp(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::state::XmrigProxy,
        state_xmrig: &Xmrig,
        path: &Path,
    ) {
        info!("XMRig-Proxy | Attempting to restart...");
        lock2!(helper, xmrig_proxy).state = ProcessState::Middle;
        lock2!(helper, xmrig_proxy).signal = ProcessSignal::Restart;

        let helper = Arc::clone(helper);
        let state = state.clone();
        let state_xmrig = state_xmrig.clone();
        let path = path.to_path_buf();
        // This thread lives to wait, start xmrig_proxy then die.
        thread::spawn(move || {
            while lock2!(helper, xmrig_proxy).state != ProcessState::Waiting {
                warn!("XMRig_proxy | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("XMRig-Proxy | Old process seems dead, starting new one!");
            Self::start_xp(&helper, &state, &state_xmrig, &path);
        });
        info!("XMRig-Proxy | Restart ... OK");
    }
    pub fn start_xp(
        helper: &Arc<Mutex<Self>>,
        state_proxy: &crate::disk::state::XmrigProxy,
        state_xmrig: &Xmrig,
        path: &Path,
    ) {
        lock2!(helper, xmrig_proxy).state = ProcessState::Middle;

        let args = Self::build_xp_args(helper, state_proxy);
        // Print arguments & user settings to console
        crate::disk::print_dash(&format!("XMRig-Proxy | Launch arguments: {:#?}", args));
        info!("XMRig-Proxy | Using path: [{}]", path.display());

        // Spawn watchdog thread
        let process = Arc::clone(&lock!(helper).xmrig_proxy);
        let gui_api = Arc::clone(&lock!(helper).gui_api_xp);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xp);
        let process_xvb = Arc::clone(&lock!(helper).xvb);
        let process_xmrig = Arc::clone(&lock!(helper).xmrig);
        let path = path.to_path_buf();
        let token = state_proxy.token.clone();
        let state_xmrig = state_xmrig.clone();
        let redirect_xmrig = state_proxy.redirect_local_xmrig;
        let pub_api_xvb = Arc::clone(&lock!(helper).pub_api_xvb);
        let gui_api_xmrig = Arc::clone(&lock!(helper).gui_api_xmrig);
        thread::spawn(move || {
            Self::spawn_xp_watchdog(
                &process,
                &gui_api,
                &pub_api,
                args,
                path,
                &token,
                &state_xmrig,
                redirect_xmrig,
                process_xvb,
                process_xmrig,
                &pub_api_xvb,
                &gui_api_xmrig,
            );
        });
    }
    #[tokio::main]
    #[allow(clippy::await_holding_lock)]
    #[allow(clippy::too_many_arguments)]
    async fn spawn_xp_watchdog(
        process: &Arc<Mutex<Process>>,
        gui_api: &Arc<Mutex<PubXmrigProxyApi>>,
        pub_api: &Arc<Mutex<PubXmrigProxyApi>>,
        args: Vec<String>,
        path: std::path::PathBuf,
        token_proxy: &str,
        state_xmrig: &Xmrig,
        xmrig_redirect: bool,
        process_xvb: Arc<Mutex<Process>>,
        process_xmrig: Arc<Mutex<Process>>,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
        gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
    ) {
        lock!(process).start = Instant::now();
        // spawn pty
        debug!("XMRig-Proxy | Creating PTY...");
        let pty = portable_pty::native_pty_system();
        let pair = pty
            .openpty(portable_pty::PtySize {
                rows: 100,
                cols: 1000,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        // 4. Spawn PTY read thread
        debug!("XMRig-Proxy | Spawning PTY read thread...");
        let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);
        spawn(enc!((pub_api_xvb, output_parse, output_pub) async move {
            Self::read_pty_xp(output_parse, output_pub, reader, process_xvb, &pub_api_xvb).await;
        }));
        // 1b. Create command
        debug!("XMRig-Proxy | Creating command...");
        let mut cmd = portable_pty::cmdbuilder::CommandBuilder::new(path.clone());
        cmd.args(args);
        cmd.cwd(path.as_path().parent().unwrap());
        // 1c. Create child
        debug!("XMRig-Proxy | Creating child...");
        let child_pty = arc_mut!(pair.slave.spawn_command(cmd).unwrap());
        drop(pair.slave);
        let mut stdin = pair.master.take_writer().unwrap();
        // to refactor to let user use his own ports
        let api_summary_xp = XMRIG_PROXY_SUMMARY_URL;
        let api_config_xmrig = XMRIG_CONFIG_URL;

        // set state
        let client = Client::new();
        lock!(process).state = ProcessState::NotMining;
        lock!(process).signal = ProcessSignal::None;
        // reset stats
        let node = lock!(pub_api).node.to_string();
        *lock!(pub_api) = PubXmrigProxyApi::new();
        *lock!(gui_api) = PubXmrigProxyApi::new();
        lock!(gui_api).node = node;
        // loop
        let start = lock!(process).start;
        debug!("Xmrig-Proxy Watchdog | enabling verbose mode");
        #[cfg(target_os = "windows")]
        if let Err(e) = write!(stdin, "v\r\n") {
            error!("P2Pool Watchdog | STDIN error: {}", e);
        }
        #[cfg(target_family = "unix")]
        if let Err(e) = writeln!(stdin, "v") {
            error!("P2Pool Watchdog | STDIN error: {}", e);
        }
        debug!("Xmrig-Proxy Watchdog | checking connections");
        #[cfg(target_os = "windows")]
        if let Err(e) = write!(stdin, "c\r\n") {
            error!("P2Pool Watchdog | STDIN error: {}", e);
        }
        #[cfg(target_family = "unix")]
        if let Err(e) = writeln!(stdin, "c") {
            error!("P2Pool Watchdog | STDIN error: {}", e);
        }
        info!("XMRig-Proxy | Entering watchdog mode... woof!");
        loop {
            let now = Instant::now();
            debug!("XMRig-Proxy Watchdog | ----------- Start of loop -----------");
            // check state
            if check_died(
                &child_pty,
                &mut lock!(process),
                &start,
                &mut lock!(gui_api).output,
            ) {
                break;
            }
            // check signal
            if signal_end(process, &child_pty, &start, &mut lock!(gui_api).output) {
                break;
            }
            // check user input
            check_user_input(process, &mut stdin);
            // get data output/api

            // Check if logs need resetting
            debug!("XMRig Watchdog | Attempting GUI log reset check");
            {
                let mut lock = lock!(gui_api);
                Self::check_reset_gui_output(&mut lock.output, ProcessName::XmrigProxy);
            }
            // Always update from output
            // todo: check difference with xmrig
            debug!("XMRig Watchdog | Starting [update_from_output()]");
            PubXmrigProxyApi::update_from_output(
                pub_api,
                &output_pub,
                &output_parse,
                start.elapsed(),
                process,
            );
            // update data from api
            debug!("XMRig-Proxy Watchdog | Attempting HTTP API request...");
            match PrivXmrigProxyApi::request_xp_api(&client, api_summary_xp, token_proxy).await {
                Ok(priv_api) => {
                    debug!("XMRig-Proxy Watchdog | HTTP API request OK, attempting [update_from_priv()]");
                    PubXmrigProxyApi::update_from_priv(pub_api, priv_api);
                }
                Err(err) => {
                    warn!(
                        "XMRig-Proxy Watchdog | Could not send HTTP API request to: {}\n{}",
                        api_summary_xp, err
                    );
                }
            }
            // update xmrig to use xmrig-proxy if option enabled and local xmrig alive
            if xmrig_redirect
                && lock!(gui_api_xmrig).node != XvbNode::XmrigProxy.to_string()
                && (lock!(process_xmrig).state == ProcessState::Alive
                    || lock!(process_xmrig).state == ProcessState::NotMining)
            {
                info!("redirect local xmrig instance to xmrig-proxy");
                if let Err(err) = update_xmrig_config(
                    &client,
                    api_config_xmrig,
                    &state_xmrig.token,
                    &XvbNode::XmrigProxy,
                    "",
                    GUPAX_VERSION_UNDERSCORE,
                )
                .await
                {
                    // show to console error about updating xmrig config
                    warn!("XMRig-Proxy Process | Failed request HTTP API Xmrig");
                    output_console(
                        &mut lock!(gui_api).output,
                        &format!(
                            "Failure to update xmrig config with HTTP API.\nError: {}",
                            err
                        ),
                        ProcessName::XmrigProxy,
                    );
                } else {
                    lock!(gui_api_xmrig).node = XvbNode::XmrigProxy.to_string();
                    debug!("XMRig-Proxy Process | mining on Xmrig-Proxy pool");
                }
            }
            // do not use more than 1 second for the loop
            sleep_end_loop(now, ProcessName::XmrigProxy).await;
        }

        // 5. If loop broke, we must be done here.
        info!("XMRig-Proxy Watchdog | Watchdog thread exiting... Goodbye!");
        // sleep
    }
}
#[derive(Debug, Clone)]
pub struct PubXmrigProxyApi {
    pub output: String,
    pub uptime: Duration,
    pub accepted: u32,
    pub rejected: u32,
    pub hashrate_1m: f32,
    pub hashrate_10m: f32,
    pub hashrate_1h: f32,
    pub hashrate_12h: f32,
    pub hashrate_24h: f32,
    pub node: String,
}

impl Default for PubXmrigProxyApi {
    fn default() -> Self {
        Self::new()
    }
}
impl PubXmrigProxyApi {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            uptime: Duration::from_secs(0),
            accepted: 0,
            rejected: 0,
            hashrate_1m: 0.0,
            hashrate_10m: 0.0,
            hashrate_1h: 0.0,
            hashrate_12h: 0.0,
            hashrate_24h: 0.0,
            node: UNKNOWN_DATA.to_string(),
        }
    }
    pub fn update_from_output(
        public: &Arc<Mutex<Self>>,
        output_parse: &Arc<Mutex<String>>,
        output_pub: &Arc<Mutex<String>>,
        elapsed: std::time::Duration,
        process: &Arc<Mutex<Process>>,
    ) {
        // 1. Take the process's current output buffer and combine it with Pub (if not empty)
        let mut output_pub = lock!(output_pub);

        {
            let mut public = lock!(public);
            if !output_pub.is_empty() {
                public.output.push_str(&std::mem::take(&mut *output_pub));
            }
            // Update uptime
            public.uptime = elapsed;
        }

        // 2. Check for "new job"/"no active...".
        let mut output_parse = lock!(output_parse);
        if XMRIG_REGEX.new_job.is_match(&output_parse)
            || XMRIG_REGEX.valid_conn.is_match(&output_parse)
        {
            lock!(process).state = ProcessState::Alive;
        } else if XMRIG_REGEX.timeout.is_match(&output_parse)
            || XMRIG_REGEX.invalid_conn.is_match(&output_parse)
            || XMRIG_REGEX.error.is_match(&output_parse)
        {
            lock!(process).state = ProcessState::NotMining;
        }
        // 3. Throw away [output_parse]
        output_parse.clear();
        drop(output_parse);
    }
    // same method as PubXmrigApi, why not make a trait ?
    pub fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        let output = std::mem::take(&mut gui_api.output);
        let node = std::mem::take(&mut gui_api.node);
        let buf = std::mem::take(&mut pub_api.output);
        *gui_api = Self {
            output,
            node,
            ..std::mem::take(pub_api)
        };
        if !buf.is_empty() {
            gui_api.output.push_str(&buf);
        }
    }
    fn update_from_priv(public: &Arc<Mutex<Self>>, private: PrivXmrigProxyApi) {
        let mut public = lock!(public);
        *public = Self {
            accepted: private.results.accepted,
            rejected: private.results.rejected,
            hashrate_1m: private.hashrate.total[0],
            hashrate_10m: private.hashrate.total[1],
            hashrate_1h: private.hashrate.total[2],
            hashrate_12h: private.hashrate.total[3],
            hashrate_24h: private.hashrate.total[4],
            ..std::mem::take(&mut *public)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PrivXmrigProxyApi {
    hashrate: HashrateProxy,
    miners: Miners,
    results: Results,
}

#[derive(Deserialize, Serialize)]
struct Results {
    accepted: u32,
    rejected: u32,
}

#[derive(Deserialize, Serialize)]
struct HashrateProxy {
    total: [f32; 6],
}

#[derive(Deserialize, Serialize)]
struct Miners {
    now: u16,
    max: u16,
}
impl PrivXmrigProxyApi {
    #[inline]
    // Send an HTTP request to XMRig's API, serialize it into [Self] and return it
    async fn request_xp_api(
        client: &Client,
        api_uri: &str,
        token: &str,
    ) -> std::result::Result<Self, anyhow::Error> {
        let request = client
            .get(api_uri)
            .header(AUTHORIZATION, ["Bearer ", token].concat());
        let mut private = request
            .timeout(std::time::Duration::from_millis(5000))
            .send()
            .await?
            .json::<PrivXmrigProxyApi>()
            .await?;
        // every hashrate value of xmrig-proxy is in kH/s, so we put convert it into H/s
        for h in &mut private.hashrate.total {
            *h *= 1000.0
        }
        Ok(private)
    }
}
