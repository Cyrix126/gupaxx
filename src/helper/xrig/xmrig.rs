use crate::constants::*;
use crate::helper::xrig::update_xmrig_config;
use crate::helper::{arc_mut, check_died, check_user_input, sleep, sleep_end_loop, Process};
use crate::helper::{Helper, ProcessName, ProcessSignal, ProcessState};
use crate::helper::{PubXvbApi, XvbNode};
use crate::miscs::output_console;
use crate::regex::{contains_error, contains_usepool, detect_new_node_xmrig, XMRIG_REGEX};
use crate::utils::human::HumanNumber;
use crate::utils::sudo::SudoState;
use enclose::enclose;
use log::*;
use portable_pty::Child;
use readable::num::Unsigned;
use readable::up::Uptime;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    fmt::Write,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    thread,
    time::*,
};
use tokio::spawn;

use super::Hashrate;

impl Helper {
    #[cold]
    #[inline(never)]
    pub async fn read_pty_xmrig(
        output_parse: Arc<Mutex<String>>,
        output_pub: Arc<Mutex<String>>,
        reader: Box<dyn std::io::Read + Send>,
        process_xvb: Arc<Mutex<Process>>,
        process_xp: Arc<Mutex<Process>>,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
    ) {
        use std::io::BufRead;
        let mut stdout = std::io::BufReader::new(reader).lines();

        // Run a ANSI escape sequence filter for the first few lines.
        let mut i = 0;
        while let Some(Ok(line)) = stdout.next() {
            let line = strip_ansi_escapes::strip_str(line);
            if let Err(e) = writeln!(output_parse.lock().unwrap(), "{}", line) {
                error!("XMRig PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(output_pub.lock().unwrap(), "{}", line) {
                error!("XMRig PTY Pub | Output error: {}", e);
            }
            if i > 13 {
                break;
            } else {
                i += 1;
            }
        }

        while let Some(Ok(line)) = stdout.next() {
            // need to verify if node still working
            // for that need to catch "connect error"
            // only check if xvb process is used and xmrig-proxy is not.
            if process_xvb.lock().unwrap().is_alive() && !process_xp.lock().unwrap().is_alive() {
                if contains_error(&line) {
                    let current_node = pub_api_xvb.lock().unwrap().current_node;
                    if let Some(current_node) = current_node {
                        // updating current node to None, will stop sending signal of FailedNode until new node is set
                        // send signal to update node.
                        warn!("XMRig PTY Parse | node is offline, sending signal to update nodes.");
                        // update nodes only if we were not mining on p2pool.
                        // if xmrig stop, xvb will react in any case.
                        if current_node != XvbNode::P2pool {
                            process_xvb.lock().unwrap().signal =
                                ProcessSignal::UpdateNodes(current_node);
                        }
                        pub_api_xvb.lock().unwrap().current_node = None;
                    }
                }
                if contains_usepool(&line) {
                    info!("XMRig PTY Parse | new pool detected");
                    // need to update current node because it was updated.
                    // if custom node made by user, it is not supported because algo is deciding which node to use.
                    let node = detect_new_node_xmrig(&line);
                    if node.is_none() {
                        error!("XMRig PTY Parse | node is not understood, switching to backup.");
                        // update with default will choose which XvB to prefer. Will update XvB to use p2pool.
                        process_xvb.lock().unwrap().signal =
                            ProcessSignal::UpdateNodes(XvbNode::default());
                    }
                    pub_api_xvb.lock().unwrap().current_node = node;
                }
            }
            //			println!("{}", line); // For debugging.
            if let Err(e) = writeln!(output_parse.lock().unwrap(), "{}", line) {
                error!("XMRig PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(output_pub.lock().unwrap(), "{}", line) {
                error!("XMRig PTY Pub | Output error: {}", e);
            }
        }
    }
    //---------------------------------------------------------------------------------------------------- XMRig specific, most functions are very similar to P2Pool's
    #[cold]
    #[inline(never)]
    // If processes are started with [sudo] on macOS, they must also
    // be killed with [sudo] (even if I have a direct handle to it as the
    // parent process...!). This is only needed on macOS, not Linux.
    fn sudo_kill(pid: u32, sudo: &Arc<Mutex<SudoState>>) -> bool {
        // Spawn [sudo] to execute [kill] on the given [pid]
        let mut child = std::process::Command::new("sudo")
            .args(["--stdin", "kill", "-9", &pid.to_string()])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        // Write the [sudo] password to STDIN.
        let mut stdin = child.stdin.take().unwrap();
        use std::io::Write;
        if let Err(e) = writeln!(stdin, "{}\n", sudo.lock().unwrap().pass) {
            error!("Sudo Kill | STDIN error: {}", e);
        }

        // Return exit code of [sudo/kill].
        child.wait().unwrap().success()
    }

    #[cold]
    #[inline(never)]
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_xmrig(helper: &Arc<Mutex<Self>>) {
        info!("XMRig | Attempting to stop...");
        helper.lock().unwrap().xmrig.lock().unwrap().signal = ProcessSignal::Stop;
        helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;
        let gui_api = Arc::clone(&helper.lock().unwrap().gui_api_xmrig);
        let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_xmrig);
        *pub_api.lock().unwrap() = PubXmrigApi::new();
        *gui_api.lock().unwrap() = PubXmrigApi::new();
    }

    #[cold]
    #[inline(never)]
    // The "restart frontend" to a "frontend" function.
    // Basically calls to kill the current xmrig, waits a little, then starts the below function in a a new thread, then exit.
    pub fn restart_xmrig(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::state::Xmrig,
        path: &Path,
        sudo: Arc<Mutex<SudoState>>,
    ) {
        info!("XMRig | Attempting to restart...");
        helper.lock().unwrap().xmrig.lock().unwrap().signal = ProcessSignal::Restart;
        helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;

        let helper = Arc::clone(helper);
        let state = state.clone();
        let path = path.to_path_buf();
        // This thread lives to wait, start xmrig then die.
        thread::spawn(move || {
            while helper.lock().unwrap().xmrig.lock().unwrap().state != ProcessState::Waiting {
                warn!("XMRig | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("XMRig | Old process seems dead, starting new one!");
            Self::start_xmrig(&helper, &state, &path, sudo);
        });
        info!("XMRig | Restart ... OK");
    }

    #[cold]
    #[inline(never)]
    pub fn start_xmrig(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::state::Xmrig,
        path: &Path,
        sudo: Arc<Mutex<SudoState>>,
    ) {
        helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;

        let (args, api_ip_port) = Self::build_xmrig_args_and_mutate_img(helper, state, path);
        // Print arguments & user settings to console
        crate::disk::print_dash(&format!("XMRig | Launch arguments: {:#?}", args));
        info!("XMRig | Using path: [{}]", path.display());

        // Spawn watchdog thread
        let process = Arc::clone(&helper.lock().unwrap().xmrig);
        let gui_api = Arc::clone(&helper.lock().unwrap().gui_api_xmrig);
        let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_xmrig);
        let process_xvb = Arc::clone(&helper.lock().unwrap().xvb);
        let process_xp = Arc::clone(&helper.lock().unwrap().xmrig_proxy);
        let path = path.to_path_buf();
        let token = state.token.clone();
        let img_xmrig = Arc::clone(&helper.lock().unwrap().img_xmrig);
        let pub_api_xvb = Arc::clone(&helper.lock().unwrap().pub_api_xvb);
        thread::spawn(move || {
            Self::spawn_xmrig_watchdog(
                process,
                gui_api,
                pub_api,
                args,
                path,
                sudo,
                api_ip_port,
                &token,
                process_xvb,
                process_xp,
                &img_xmrig,
                &pub_api_xvb,
            );
        });
    }

    #[cold]
    #[inline(never)]
    // Takes in some [State/Xmrig] and parses it to build the actual command arguments.
    // Returns the [Vec] of actual arguments, and mutates the [ImgXmrig] for the main GUI thread
    // It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
    pub fn build_xmrig_args_and_mutate_img(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::state::Xmrig,
        path: &std::path::Path,
    ) -> (Vec<String>, String) {
        let mut args = Vec::with_capacity(500);
        let mut api_ip = String::with_capacity(15);
        let mut api_port = String::with_capacity(5);
        let path = path.to_path_buf();
        // The actual binary we're executing is [sudo], technically
        // the XMRig path is just an argument to sudo, so add it.
        // Before that though, add the ["--prompt"] flag and set it
        // to emptyness so that it doesn't show up in the output.
        if cfg!(unix) {
            args.push(r#"--prompt="#.to_string());
            args.push("--".to_string());
            args.push(path.display().to_string());
        }

        // [Simple]
        if state.simple {
            // Build the xmrig argument
            let rig = if state.simple_rig.is_empty() {
                GUPAX_VERSION_UNDERSCORE.to_string()
            } else {
                state.simple_rig.clone()
            }; // Rig name
            args.push("--url".to_string());
            args.push("127.0.0.1:3333".to_string()); // Local P2Pool (the default)
            args.push("--threads".to_string());
            args.push(state.current_threads.to_string()); // Threads
            args.push("--user".to_string());
            args.push(rig); // Rig name
            args.push("--no-color".to_string()); // No color
            args.push("--http-host".to_string());
            args.push("127.0.0.1".to_string()); // HTTP API IP
            args.push("--http-port".to_string());
            args.push("18088".to_string()); // HTTP API Port
            if state.pause != 0 {
                args.push("--pause-on-active".to_string());
                args.push(state.pause.to_string());
            } // Pause on active
            *helper.lock().unwrap().img_xmrig.lock().unwrap() = ImgXmrig {
                threads: state.current_threads.to_string(),
                url: "127.0.0.1:3333 (Local P2Pool)".to_string(),
            };

            helper.lock().unwrap().pub_api_xmrig.lock().unwrap().node =
                "127.0.0.1:3333 (Local P2Pool)".to_string();
            api_ip = "127.0.0.1".to_string();
            api_port = "18088".to_string();

        // [Advanced]
        } else {
            // Overriding command arguments
            if !state.arguments.is_empty() {
                // This parses the input and attempts to fill out
                // the [ImgXmrig]... This is pretty bad code...
                let mut last = "";
                let lock = helper.lock().unwrap();
                let mut xmrig_image = lock.img_xmrig.lock().unwrap();
                for arg in state.arguments.split_whitespace() {
                    match last {
                        "--threads" => xmrig_image.threads = arg.to_string(),
                        "--url" => xmrig_image.url = arg.to_string(),
                        "--http-host" => {
                            api_ip = if arg == "localhost" {
                                "127.0.0.1".to_string()
                            } else {
                                arg.to_string()
                            }
                        }
                        "--http-port" => api_port = arg.to_string(),
                        _ => (),
                    }
                    args.push(if arg == "localhost" {
                        "127.0.0.1".to_string()
                    } else {
                        arg.to_string()
                    });
                    last = arg;
                }
            // Else, build the argument
            } else {
                // XMRig doesn't understand [localhost]
                let ip = if state.ip == "localhost" || state.ip.is_empty() {
                    "127.0.0.1"
                } else {
                    &state.ip
                };
                api_ip = if state.api_ip == "localhost" || state.api_ip.is_empty() {
                    "127.0.0.1".to_string()
                } else {
                    state.api_ip.to_string()
                };
                api_port = if state.api_port.is_empty() {
                    "18088".to_string()
                } else {
                    state.api_port.to_string()
                };
                let url = format!("{}:{}", ip, state.port); // Combine IP:Port into one string
                args.push("--user".to_string());
                args.push(state.address.clone()); // Wallet
                args.push("--threads".to_string());
                args.push(state.current_threads.to_string()); // Threads
                args.push("--rig-id".to_string());
                args.push(state.rig.to_string()); // Rig ID
                args.push("--url".to_string());
                args.push(url.clone()); // IP/Port
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
                if state.pause != 0 {
                    args.push("--pause-on-active".to_string());
                    args.push(state.pause.to_string());
                } // Pause on active
                *helper.lock().unwrap().img_xmrig.lock().unwrap() = ImgXmrig {
                    url: url.clone(),
                    threads: state.current_threads.to_string(),
                };
                helper.lock().unwrap().pub_api_xmrig.lock().unwrap().node = url;
            }
        }
        args.push(format!("--http-access-token={}", state.token)); // HTTP API Port
        args.push("--http-no-restricted".to_string());
        (args, format!("{}:{}", api_ip, api_port))
    }

    // We actually spawn [sudo] on Unix, with XMRig being the argument.
    #[cfg(target_family = "unix")]
    fn create_xmrig_cmd_unix(args: Vec<String>, path: PathBuf) -> portable_pty::CommandBuilder {
        let mut cmd = portable_pty::cmdbuilder::CommandBuilder::new("sudo");
        cmd.args(args);
        cmd.cwd(path.as_path().parent().unwrap());
        cmd
    }

    // Gupax should be admin on Windows, so just spawn XMRig normally.
    #[cfg(target_os = "windows")]
    fn create_xmrig_cmd_windows(args: Vec<String>, path: PathBuf) -> portable_pty::CommandBuilder {
        let mut cmd = portable_pty::cmdbuilder::CommandBuilder::new(path.clone());
        cmd.args(args);
        cmd.cwd(path.as_path().parent().unwrap());
        cmd
    }

    #[cold]
    #[inline(never)]
    // The XMRig watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
    // This isn't actually async, a tokio runtime is unfortunately needed because [Hyper] is an async library (HTTP API calls)
    #[tokio::main]
    #[allow(clippy::await_holding_lock)]
    #[allow(clippy::too_many_arguments)]
    async fn spawn_xmrig_watchdog(
        process: Arc<Mutex<Process>>,
        gui_api: Arc<Mutex<PubXmrigApi>>,
        pub_api: Arc<Mutex<PubXmrigApi>>,
        args: Vec<String>,
        path: std::path::PathBuf,
        sudo: Arc<Mutex<SudoState>>,
        mut api_ip_port: String,
        token: &str,
        process_xvb: Arc<Mutex<Process>>,
        process_xp: Arc<Mutex<Process>>,
        img_xmrig: &Arc<Mutex<ImgXmrig>>,
        pub_api_xvb: &Arc<Mutex<PubXvbApi>>,
    ) {
        // 1a. Create PTY
        debug!("XMRig | Creating PTY...");
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
        debug!("XMRig | Spawning PTY read thread...");
        let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
        let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
        let output_pub = Arc::clone(&process.lock().unwrap().output_pub);
        spawn(enclose!((pub_api_xvb, process_xp) async move {
            Self::read_pty_xmrig(output_parse, output_pub, reader, process_xvb, process_xp, &pub_api_xvb).await;
        }));
        // 1b. Create command
        debug!("XMRig | Creating command...");
        #[cfg(target_os = "windows")]
        let cmd = Self::create_xmrig_cmd_windows(args, path);
        #[cfg(target_family = "unix")]
        let cmd = Self::create_xmrig_cmd_unix(args, path);
        // 1c. Create child
        debug!("XMRig | Creating child...");
        let child_pty = arc_mut!(pair.slave.spawn_command(cmd).unwrap());
        drop(pair.slave);

        let mut stdin = pair.master.take_writer().unwrap();

        // 2. Input [sudo] pass, wipe, then drop.
        if cfg!(unix) {
            debug!("XMRig | Inputting [sudo] and wiping...");
            // a) Sleep to wait for [sudo]'s non-echo prompt (on Unix).
            // this prevents users pass from showing up in the STDOUT.
            sleep!(3000);
            if let Err(e) = writeln!(stdin, "{}", sudo.lock().unwrap().pass) {
                error!("XMRig | Sudo STDIN error: {}", e);
            };
            SudoState::wipe(&sudo);

            // b) Reset GUI STDOUT just in case.
            debug!("XMRig | Clearing GUI output...");
            gui_api.lock().unwrap().output.clear();
        }

        // 3. Set process state
        debug!("XMRig | Setting process state...");
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::NotMining;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
        drop(lock);

        // // 4. Spawn PTY read thread
        // debug!("XMRig | Spawning PTY read thread...");
        // let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
        // let output_pub = Arc::clone(&process.lock().unwrap().output_pub);
        // spawn(enclose!((pub_api_xvb) async move {
        //     Self::read_pty_xmrig(output_parse, output_pub, reader, process_xvb, &pub_api_xvb).await;
        // }));
        let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
        let output_pub = Arc::clone(&process.lock().unwrap().output_pub);

        let client = Client::new();
        let start = process.lock().unwrap().start;
        let api_uri = {
            if !api_ip_port.ends_with('/') {
                api_ip_port.push('/');
            }
            "http://".to_owned() + &api_ip_port + XMRIG_API_SUMMARY_URI
        };
        info!("XMRig | Final API URI: {}", api_uri);

        // Reset stats before loop
        *pub_api.lock().unwrap() = PubXmrigApi::new();
        *gui_api.lock().unwrap() = PubXmrigApi::new();
        // node used for process Status tab
        gui_api
            .lock()
            .unwrap()
            .node
            .clone_from(&img_xmrig.lock().unwrap().url);
        // 5. Loop as watchdog
        info!("XMRig | Entering watchdog mode... woof!");
        // needs xmrig to be in belownormal priority or else Gupaxx will be in trouble if it does not have enough cpu time.
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            std::process::Command::new("cmd")
                .creation_flags(0x08000000)
                .args(["/c", "wmic"])
                .args([
                    "process",
                    "where",
                    "name='xmrig.exe'",
                    "CALL",
                    "setpriority",
                    "below normal",
                ])
                .spawn()
                .expect("failure to execute command wmic");
        }
        loop {
            // Set timer
            let now = Instant::now();
            debug!("XMRig Watchdog | ----------- Start of loop -----------");

            // Check if the process secretly died without us knowing :)
            if check_died(
                &child_pty,
                &mut process.lock().unwrap(),
                &start,
                &mut gui_api.lock().unwrap().output,
            ) {
                break;
            }
            // Stop on [Stop/Restart] SIGNAL
            if Self::xmrig_signal_end(
                &mut process.lock().unwrap(),
                &child_pty,
                &start,
                &mut gui_api.lock().unwrap().output,
                &sudo,
            ) {
                break;
            }
            // Check vector of user input
            check_user_input(&process, &mut stdin);
            // Check if logs need resetting
            debug!("XMRig Watchdog | Attempting GUI log reset check");
            {
                let mut lock = gui_api.lock().unwrap();
                Self::check_reset_gui_output(&mut lock.output, ProcessName::Xmrig);
            }
            // Always update from output
            debug!("XMRig Watchdog | Starting [update_from_output()]");
            PubXmrigApi::update_from_output(
                &pub_api,
                &output_pub,
                &output_parse,
                start.elapsed(),
                &process,
            );
            // Send an HTTP API request
            debug!("XMRig Watchdog | Attempting HTTP API request...");
            match PrivXmrigApi::request_xmrig_api(&client, &api_uri, token).await {
                Ok(priv_api) => {
                    debug!("XMRig Watchdog | HTTP API request OK, attempting [update_from_priv()]");
                    PubXmrigApi::update_from_priv(&pub_api, priv_api);
                }
                Err(err) => {
                    warn!(
                        "XMRig Watchdog | Could not send HTTP API request to: {}\n{}",
                        api_uri, err
                    );
                }
            }
            // if mining on proxy and proxy is not alive, switch back to p2pool node
            if gui_api.lock().unwrap().node == XvbNode::XmrigProxy.to_string()
                && !process_xp.lock().unwrap().is_alive()
            {
                info!("XMRig Process |  redirect xmrig to p2pool since XMRig-Proxy is not alive anymore");
                if let Err(err) = update_xmrig_config(
                    &client,
                    XMRIG_CONFIG_URL,
                    token,
                    &XvbNode::P2pool,
                    "",
                    GUPAX_VERSION_UNDERSCORE,
                )
                .await
                {
                    // show to console error about updating xmrig config
                    warn!("XMRig Process | Failed request HTTP API Xmrig");
                    output_console(
                        &mut gui_api.lock().unwrap().output,
                        &format!(
                            "Failure to update xmrig config with HTTP API.\nError: {}",
                            err
                        ),
                        ProcessName::Xmrig,
                    );
                } else {
                    gui_api.lock().unwrap().node = XvbNode::P2pool.to_string();
                    debug!("XMRig Process | mining on P2Pool pool");
                }
            }
            // Sleep (only if 900ms hasn't passed)
            sleep_end_loop(now, ProcessName::Xmrig).await;
        }

        // 5. If loop broke, we must be done here.
        info!("XMRig Watchdog | Watchdog thread exiting... Goodbye!");
    }
    fn xmrig_signal_end(
        process: &mut Process,
        child_pty: &Arc<Mutex<Box<dyn Child + Sync + Send>>>,
        start: &Instant,
        gui_api_output_raw: &mut String,
        sudo: &Arc<Mutex<SudoState>>,
    ) -> bool {
        let signal = process.signal;
        if signal == ProcessSignal::Stop || signal == ProcessSignal::Restart {
            debug!("XMRig Watchdog | Stop/Restart SIGNAL caught");
            // macOS requires [sudo] again to kill [XMRig]
            if cfg!(target_os = "macos") {
                // If we're at this point, that means the user has
                // entered their [sudo] pass again, after we wiped it.
                // So, we should be able to find it in our [Arc<Mutex<SudoState>>].
                Self::sudo_kill(child_pty.lock().unwrap().process_id().unwrap(), sudo);
                // And... wipe it again (only if we're stopping full).
                // If we're restarting, the next start will wipe it for us.
                if signal != ProcessSignal::Restart {
                    SudoState::wipe(sudo);
                }
            } else if let Err(e) = child_pty.lock().unwrap().kill() {
                error!("XMRig Watchdog | Kill error: {}", e);
            }
            let exit_status = match child_pty.lock().unwrap().wait() {
                Ok(e) => {
                    if e.success() {
                        if process.signal == ProcessSignal::Stop {
                            process.state = ProcessState::Dead;
                        }
                        "Successful"
                    } else {
                        if process.signal == ProcessSignal::Stop {
                            process.state = ProcessState::Failed;
                        }
                        "Failed"
                    }
                }
                _ => {
                    if process.signal == ProcessSignal::Stop {
                        process.state = ProcessState::Failed;
                    }
                    "Unknown Error"
                }
            };
            let uptime = Uptime::from(start.elapsed());
            info!(
                "XMRig | Stopped ... Uptime was: [{}], Exit status: [{}]",
                uptime, exit_status
            );
            if let Err(e) = writeln!(
                gui_api_output_raw,
                "{}\nXMRig stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                HORI_CONSOLE, uptime, exit_status, HORI_CONSOLE
            ) {
                error!(
                    "XMRig Watchdog | GUI Uptime/Exit status write failed: {}",
                    e
                );
            }
            match process.signal {
                ProcessSignal::Stop => process.signal = ProcessSignal::None,
                ProcessSignal::Restart => process.state = ProcessState::Waiting,
                _ => (),
            }
            debug!("XMRig Watchdog | Stop/Restart SIGNAL done, breaking");
            return true;
        }
        false
    }
}
//---------------------------------------------------------------------------------------------------- [ImgXmrig]
#[derive(Debug, Clone)]
pub struct ImgXmrig {
    pub threads: String,
    pub url: String,
}

impl Default for ImgXmrig {
    fn default() -> Self {
        Self::new()
    }
}

impl ImgXmrig {
    pub fn new() -> Self {
        Self {
            threads: "???".to_string(),
            url: "???".to_string(),
        }
    }
}

//---------------------------------------------------------------------------------------------------- Public XMRig API
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PubXmrigApi {
    pub output: String,
    pub uptime: Duration,
    pub worker_id: String,
    pub resources: String,
    pub hashrate: String,
    pub diff: String,
    pub accepted: String,
    pub rejected: String,
    pub hashrate_raw: f32,
    pub hashrate_raw_1m: f32,
    pub hashrate_raw_15m: f32,
    pub node: String,
}

impl Default for PubXmrigApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PubXmrigApi {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            uptime: Duration::from_secs(0),
            worker_id: UNKNOWN_DATA.to_string(),
            resources: UNKNOWN_DATA.to_string(),
            hashrate: UNKNOWN_DATA.to_string(),
            diff: UNKNOWN_DATA.to_string(),
            accepted: UNKNOWN_DATA.to_string(),
            rejected: UNKNOWN_DATA.to_string(),
            hashrate_raw: 0.0,
            hashrate_raw_1m: 0.0,
            hashrate_raw_15m: 0.0,
            node: UNKNOWN_DATA.to_string(),
        }
    }

    #[inline]
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

    // This combines the buffer from the PTY thread [output_pub]
    // with the actual [PubApiXmrig] output field.
    pub fn update_from_output(
        public: &Arc<Mutex<Self>>,
        output_parse: &Arc<Mutex<String>>,
        output_pub: &Arc<Mutex<String>>,
        elapsed: std::time::Duration,
        process: &Arc<Mutex<Process>>,
    ) {
        // 1. Take the process's current output buffer and combine it with Pub (if not empty)
        let mut output_pub = output_pub.lock().unwrap();

        {
            let mut public = public.lock().unwrap();
            if !output_pub.is_empty() {
                public.output.push_str(&std::mem::take(&mut *output_pub));
            }
            // Update uptime
            public.uptime = elapsed;
        }

        // 2. Check for "new job"/"no active...".
        let mut output_parse = output_parse.lock().unwrap();
        if XMRIG_REGEX.new_job.is_match(&output_parse) {
            process.lock().unwrap().state = ProcessState::Alive;
        } else if XMRIG_REGEX.not_mining.is_match(&output_parse) {
            process.lock().unwrap().state = ProcessState::NotMining;
        }

        // 3. Throw away [output_parse]
        output_parse.clear();
        drop(output_parse);
    }

    // Formats raw private data into ready-to-print human readable version.
    fn update_from_priv(public: &Arc<Mutex<Self>>, private: PrivXmrigApi) {
        let mut public = public.lock().unwrap();
        let hashrate_raw = match private.hashrate.total.first() {
            Some(Some(h)) => *h,
            _ => 0.0,
        };
        let hashrate_raw_1m = match private.hashrate.total.get(1) {
            Some(Some(h)) => *h,
            _ => 0.0,
        };
        let hashrate_raw_15m = match private.hashrate.total.last() {
            Some(Some(h)) => *h,
            _ => 0.0,
        };

        *public = Self {
            worker_id: private.worker_id,
            resources: HumanNumber::from_load(private.resources.load_average).to_string(),
            hashrate: HumanNumber::from_hashrate(private.hashrate.total).to_string(),
            diff: Unsigned::from(private.connection.diff as usize).to_string(),
            accepted: Unsigned::from(private.connection.accepted as usize).to_string(),
            rejected: Unsigned::from(private.connection.rejected as usize).to_string(),
            hashrate_raw,
            hashrate_raw_1m,
            hashrate_raw_15m,
            ..std::mem::take(&mut *public)
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private XMRig API
// This matches to some JSON stats in the HTTP call [summary],
// e.g: [wget -qO- localhost:18085/1/summary].
// XMRig doesn't initialize stats at 0 (or 0.0) and instead opts for [null]
// which means some elements need to be wrapped in an [Option] or else serde will [panic!].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivXmrigApi {
    worker_id: String,
    resources: Resources,
    connection: Connection,
    hashrate: Hashrate,
}

impl PrivXmrigApi {
    #[inline]
    // Send an HTTP request to XMRig's API, serialize it into [Self] and return it
    async fn request_xmrig_api(
        client: &Client,
        api_uri: &str,
        token: &str,
    ) -> std::result::Result<Self, anyhow::Error> {
        let request = client
            .get(api_uri)
            .header(AUTHORIZATION, ["Bearer ", token].concat());
        Ok(request
            .timeout(std::time::Duration::from_millis(5000))
            .send()
            .await?
            .json()
            .await?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Resources {
    load_average: [Option<f32>; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Connection {
    diff: u128,
    accepted: u128,
    rejected: u128,
}
