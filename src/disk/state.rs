use anyhow::Result;
use rand::{Rng, distr::Alphanumeric, rng};
use strum::{EnumCount, EnumIter};

use super::*;
use crate::{
    app::panels::middle::common::list_poolnode::PoolNode,
    components::node::RemoteNode,
    disk::status::*,
    helper::{Helper, ProcessName, node::ImgNode, p2pool::ImgP2pool, xrig::xmrig_proxy::ImgProxy},
};
//---------------------------------------------------------------------------------------------------- [State] Impl
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let max_threads = benri::threads!() as u16;
        let current_threads = if max_threads == 1 { 1 } else { max_threads / 2 };
        Self {
            status: Status::default(),
            gupax: Gupax::default(),
            p2pool: P2pool::default(),
            xmrig: Xmrig::with_threads(max_threads, current_threads),
            xvb: Xvb::default(),
            xmrig_proxy: XmrigProxy::default(),
            node: Node::default(),
            version: arc_mut!(Version::default()),
        }
    }

    pub fn update_absolute_path(&mut self) -> Result<(), TomlError> {
        self.gupax.absolute_p2pool_path = into_absolute_path(self.gupax.p2pool_path.clone())?;
        self.gupax.absolute_xmrig_path = into_absolute_path(self.gupax.xmrig_path.clone())?;
        self.gupax.absolute_xp_path = into_absolute_path(self.gupax.xmrig_proxy_path.clone())?;
        self.gupax.absolute_node_path = into_absolute_path(self.gupax.node_path.clone())?;
        Ok(())
    }

    // Convert [&str] to [State]
    pub fn from_str(string: &str) -> Result<Self, TomlError> {
        match toml::de::from_str(string) {
            Ok(state) => {
                info!("State | Parse ... OK");
                print_dash(string);
                Ok(state)
            }
            Err(err) => {
                warn!("State | String -> State ... FAIL ... {}", err);
                Err(TomlError::Deserialize(err))
            }
        }
    }

    // Convert [State] to [String]
    pub fn to_string(&self) -> Result<String, TomlError> {
        match toml::ser::to_string(self) {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("State | Couldn't serialize default file: {}", e);
                Err(TomlError::Serialize(e))
            }
        }
    }

    // Combination of multiple functions:
    //   1. Attempt to read file from path into [String]
    //      |_ Create a default file if not found
    //   2. Deserialize [String] into a proper [Struct]
    //      |_ Attempt to merge if deserialization fails
    pub fn get(path: &PathBuf) -> Result<Self, TomlError> {
        // Read
        let file = File::State;
        let string = match read_to_string(file, path) {
            Ok(string) => string,
            // Create
            _ => {
                Self::create_new(path)?;
                read_to_string(file, path)?
            }
        };
        // Deserialize, attempt merge if failed
        match Self::from_str(&string) {
            Ok(s) => Ok(s),
            Err(_) => {
                warn!("State | Attempting merge...");
                match Self::merge(&string) {
                    Ok(mut new) => {
                        Self::save(&mut new, path)?;
                        Ok(new)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    // Completely overwrite current [state.toml]
    // with a new default version, and return [Self].
    pub fn create_new(path: &PathBuf) -> Result<Self, TomlError> {
        info!("State | Creating new default...");
        let new = Self::new();
        let string = Self::to_string(&new)?;
        fs::write(path, string)?;
        info!("State | Write ... OK");
        Ok(new)
    }

    // Save [State] onto disk file [gupax.toml]
    pub fn save(&mut self, path: &PathBuf) -> Result<(), TomlError> {
        info!("State | Saving to disk...");
        // Convert path to absolute
        self.gupax.absolute_p2pool_path = into_absolute_path(self.gupax.p2pool_path.clone())?;
        self.gupax.absolute_xmrig_path = into_absolute_path(self.gupax.xmrig_path.clone())?;
        self.gupax.absolute_xp_path = into_absolute_path(self.gupax.xmrig_proxy_path.clone())?;
        let string = match toml::ser::to_string(&self) {
            Ok(string) => {
                info!("State | Parse ... OK");
                print_dash(&string);
                string
            }
            Err(err) => {
                error!("State | Couldn't parse TOML into string ... FAIL");
                return Err(TomlError::Serialize(err));
            }
        };
        match fs::write(path, string) {
            Ok(_) => {
                info!("State | Save ... OK");
                Ok(())
            }
            Err(err) => {
                error!("State | Couldn't overwrite TOML file ... FAIL");
                Err(TomlError::Io(err))
            }
        }
    }
    // Take [String] as input, merge it with whatever the current [default] is,
    // leaving behind old keys+values and updating [default] with old valid ones.
    pub fn merge(old: &str) -> Result<Self, TomlError> {
        let default = toml::ser::to_string(&Self::new()).unwrap();
        let new: Self = match Figment::from(Toml::string(&default))
            .merge(Toml::string(old))
            .extract()
        {
            Ok(new) => {
                info!("State | TOML merge ... OK");
                new
            }
            Err(err) => {
                error!("State | Couldn't merge default + old TOML");
                return Err(TomlError::Merge(err));
            }
        };
        Ok(new)
    }
}
//---------------------------------------------------------------------------------------------------- [State] Struct
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub status: Status,
    pub gupax: Gupax,
    pub p2pool: P2pool,
    pub xmrig: Xmrig,
    pub xmrig_proxy: XmrigProxy,
    pub xvb: Xvb,
    pub node: Node,
    pub version: Arc<Mutex<Version>>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Status {
    pub submenu: Submenu,
    pub payout_view: PayoutView,
    pub monero_enabled: bool,
    pub manual_hash: bool,
    pub hashrate: f64,
    pub hash_metric: Hash,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Gupax {
    pub simple: bool,
    pub auto: AutoEnabled,
    pub p2pool_path: String,
    pub node_path: String,
    pub xmrig_path: String,
    pub xmrig_proxy_path: String,
    pub absolute_p2pool_path: PathBuf,
    pub absolute_node_path: PathBuf,
    pub absolute_xmrig_path: PathBuf,
    pub absolute_xp_path: PathBuf,
    pub selected_width: u16,
    pub selected_height: u16,
    pub selected_scale: f32,
    pub tab: Tab,
    pub ratio: Ratio,
    pub show_processes: Vec<ProcessName>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct AutoEnabled {
    pub update: bool,
    pub bundled: bool,
    pub ask_before_quit: bool,
    pub save_before_quit: bool,
    pub processes: Vec<ProcessName>,
}
impl AutoEnabled {
    pub fn enable(&mut self, auto: &AutoStart, enable: bool) {
        match auto {
            AutoStart::Update => self.update = enable,
            AutoStart::Bundle => self.bundled = enable,
            AutoStart::AskBeforeQuit => self.ask_before_quit = enable,
            AutoStart::SaveBeforequit => self.save_before_quit = enable,
            AutoStart::Process(p) => {
                let processes = &mut self.processes;
                if !processes.iter().any(|a| a == p) && enable {
                    self.processes.push(*p);
                } else if let Some(i) = processes.iter().position(|a| a == p) {
                    if !enable {
                        processes.remove(i);
                    }
                }
            }
        }
    }
    pub fn is_enabled(&self, auto: &AutoStart) -> bool {
        match auto {
            AutoStart::Update => self.update,
            AutoStart::Bundle => self.bundled,
            AutoStart::AskBeforeQuit => self.ask_before_quit,
            AutoStart::SaveBeforequit => self.save_before_quit,
            AutoStart::Process(p) => self.processes.iter().any(|a| a == p),
        }
    }
}
#[derive(PartialEq, strum::Display, EnumCount, EnumIter)]
pub enum AutoStart {
    #[strum(to_string = "Auto-Update")]
    Update,
    Bundle,
    #[strum(to_string = "Confirm quit")]
    AskBeforeQuit,
    #[strum(to_string = "Save on exit")]
    SaveBeforequit,
    #[strum(to_string = "Auto-{0}")]
    Process(ProcessName),
}
impl AutoStart {
    pub const fn help_msg(&self) -> &str {
        match self {
            AutoStart::Update => GUPAX_AUTO_UPDATE,
            AutoStart::Bundle => GUPAX_BUNDLED_UPDATE,
            AutoStart::AskBeforeQuit => GUPAX_ASK_BEFORE_QUIT,
            AutoStart::SaveBeforequit => GUPAX_SAVE_BEFORE_QUIT,
            AutoStart::Process(p) => p.msg_auto_help(),
        }
    }
    // todo: generate as const with all process in middle ?
    // Would necessities unstable feature https://github.com/rust-lang/rust/issues/87575
    pub const ALL: &[AutoStart] = &[
        AutoStart::Update,
        AutoStart::Bundle,
        AutoStart::Process(ProcessName::Node),
        AutoStart::Process(ProcessName::P2pool),
        AutoStart::Process(ProcessName::Xmrig),
        AutoStart::Process(ProcessName::XmrigProxy),
        AutoStart::Process(ProcessName::Xvb),
        AutoStart::AskBeforeQuit,
        AutoStart::SaveBeforequit,
    ];
    // non const:
    // let mut autos = AutoStart::iter().collect::<Vec<_>>();
    // // remove ProcessName default
    // autos.remove(AutoStart::COUNT - 1);
    // // insert ProcessName before AskBeforeQuit
    // let before_quit_index = autos
    //     .iter()
    //     .position(|a| *a == AutoStart::AskBeforeQuit)
    //     .expect("Before quit should be in iter");
    // ProcessName::iter()
    //     .rev()
    //     .for_each(|p| autos.insert(before_quit_index, AutoStart::Process(p)));
    // autos
}
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct P2pool {
    pub simple: bool,
    pub local_node: bool,
    pub mini: bool,
    pub auto_ping: bool,
    pub auto_select: bool,
    pub backup_host: bool,
    pub out_peers: u16,
    pub in_peers: u16,
    pub log_level: u16,
    pub node: String,
    pub arguments: String,
    pub address: String,
    pub name: String,
    pub ip: String,
    pub rpc: String,
    pub zmq: String,
    pub stratum_port: u16,
    pub selected_node: SelectedPoolNode,
    pub prefer_local_node: bool,
    pub console_height: u32,
}

// compatible for P2Pool and Xmrig/Proxy
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct SelectedPoolNode {
    pub index: usize,
    pub name: String,
    pub ip: String,
    pub rpc: String,
    pub zmq_rig: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Node {
    pub simple: bool,
    pub api_ip: String,
    pub api_port: String,
    pub out_peers: u16,
    pub in_peers: u16,
    pub log_level: u16,
    pub arguments: String,
    pub zmq_ip: String,
    pub zmq_port: String,
    pub pruned: bool,
    pub dns_blocklist: bool,
    pub disable_dns_checkpoint: bool,
    pub path_db: String,
    pub full_memory: bool,
    pub console_height: u32,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            simple: true,
            api_ip: String::from("127.0.0.1"),
            api_port: 18081.to_string(),
            out_peers: 8,
            in_peers: 16,
            log_level: 0,
            arguments: String::new(),
            zmq_ip: String::from("127.0.0.1"),
            zmq_port: 18083.to_string(),
            pruned: true,
            dns_blocklist: true,
            disable_dns_checkpoint: true,
            path_db: String::new(),
            full_memory: false,
            console_height: APP_DEFAULT_CONSOLE_HEIGHT,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Xmrig {
    pub simple: bool,
    pub pause: u16,
    pub simple_rig: String,
    pub arguments: String,
    pub tls: bool,
    pub keepalive: bool,
    pub max_threads: u16,
    pub current_threads: u16,
    pub address: String,
    pub api_ip: String,
    pub api_port: String,
    pub name: String,
    pub rig: String,
    pub ip: String,
    pub port: String,
    pub selected_pool: SelectedPoolNode,
    pub token: String,
    pub console_height: u32,
}

// present for future.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct XmrigProxy {
    pub simple: bool,
    pub arguments: String,
    pub simple_rig: String,
    pub tls: bool,
    pub keepalive: bool,
    pub address: String,
    pub name: String,
    pub rig: String,
    pub ip: String,
    pub port: String,
    pub api_ip: String,
    pub api_port: String,
    pub p2pool_ip: String,
    pub p2pool_port: String,
    pub selected_pool: SelectedPoolNode,
    pub token: String,
    pub redirect_local_xmrig: bool,
    pub console_height: u32,
}

impl Gupax {
    pub fn path_binary(&mut self, process: &BundledProcess) -> &mut String {
        match process {
            BundledProcess::Node => &mut self.node_path,
            BundledProcess::P2Pool => &mut self.p2pool_path,
            BundledProcess::Xmrig => &mut self.xmrig_path,
            BundledProcess::XmrigProxy => &mut self.xmrig_proxy_path,
        }
    }
}

// do not include process that are from Gupaxx
#[derive(EnumIter)]
pub enum BundledProcess {
    Node,
    P2Pool,
    Xmrig,
    XmrigProxy,
}
impl BundledProcess {
    pub fn process_name(&self) -> ProcessName {
        match self {
            BundledProcess::Node => ProcessName::Node,
            BundledProcess::P2Pool => ProcessName::P2pool,
            BundledProcess::Xmrig => ProcessName::Xmrig,
            BundledProcess::XmrigProxy => ProcessName::XmrigProxy,
        }
    }
}

impl Default for XmrigProxy {
    fn default() -> Self {
        XmrigProxy {
            simple: true,
            arguments: Default::default(),
            token: rng()
                .sample_iter(Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
            redirect_local_xmrig: true,
            address: String::with_capacity(96),
            name: "Local P2Pool".to_string(),
            rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            simple_rig: String::with_capacity(30),
            ip: "0.0.0.0".to_string(),
            port: "3355".to_string(),
            p2pool_ip: "localhost".to_string(),
            p2pool_port: "3333".to_string(),
            selected_pool: SelectedPoolNode {
                index: 0,
                name: "Local P2Pool".to_string(),
                ip: "localhost".to_string(),
                rpc: "3333".to_string(),
                zmq_rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            },
            api_ip: "localhost".to_string(),
            api_port: "18089".to_string(),
            tls: false,
            keepalive: false,
            console_height: APP_DEFAULT_CONSOLE_HEIGHT,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Xvb {
    pub simple: bool,
    pub token: String,
    pub simple_hero_mode: bool,
    pub mode: XvbMode,
    pub manual_amount_raw: f64,
    pub manual_slider_amount: f64,
    pub manual_donation_level: ManualDonationLevel,
    pub manual_donation_metric: ManualDonationMetric,
    pub p2pool_buffer: i8,
    pub use_p2pool_sidechain_hr: bool,
    pub console_height: u32,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, Default, EnumCount, EnumIter)]
pub enum XvbMode {
    #[default]
    Auto,
    Hero,
    ManualXvb,
    ManualP2pool,
    ManualDonationLevel,
}

impl Display for XvbMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Auto => "Auto",
            Self::Hero => "Hero",
            Self::ManualXvb => "Manual Xvb",
            Self::ManualP2pool => "Manual P2pool",
            Self::ManualDonationLevel => "Manual Donation Level",
        };

        write!(f, "{}", text)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ManualDonationLevel {
    #[default]
    Donor,
    DonorVIP,
    DonorWhale,
    DonorMega,
}

impl Display for ManualDonationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Donor => "Donor",
            Self::DonorVIP => "Donor VIP",
            Self::DonorWhale => "Donor Whale",
            Self::DonorMega => "Donor Mega",
        };

        write!(f, "{}", text)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ManualDonationMetric {
    #[default]
    Hash,
    Kilo,
    Mega,
}

impl Display for ManualDonationMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Hash => "H/s",
            Self::Kilo => "KH/s",
            Self::Mega => "MH/s",
        };

        write!(f, "{}", text)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Version {
    pub gupax: String,
    pub p2pool: String,
    pub xmrig: String,
}

//---------------------------------------------------------------------------------------------------- [State] Defaults
impl Default for AutoEnabled {
    fn default() -> Self {
        Self {
            update: false,
            #[cfg(feature = "bundle")]
            bundled: true,
            #[cfg(not(feature = "bundle"))]
            bundled: false,
            ask_before_quit: true,
            save_before_quit: true,
            processes: Vec::new(),
        }
    }
}
impl Default for Status {
    fn default() -> Self {
        Self {
            submenu: Submenu::default(),
            payout_view: PayoutView::default(),
            monero_enabled: false,
            manual_hash: false,
            hashrate: 1.0,
            hash_metric: Hash::default(),
        }
    }
}

impl Default for Gupax {
    fn default() -> Self {
        Self {
            simple: true,
            auto: AutoEnabled::default(),
            p2pool_path: DEFAULT_P2POOL_PATH.to_string(),
            xmrig_path: DEFAULT_XMRIG_PATH.to_string(),
            node_path: DEFAULT_NODE_PATH.to_string(),
            xmrig_proxy_path: DEFAULT_XMRIG_PROXY_PATH.to_string(),
            absolute_p2pool_path: into_absolute_path(DEFAULT_P2POOL_PATH.to_string()).unwrap(),
            absolute_xmrig_path: into_absolute_path(DEFAULT_XMRIG_PATH.to_string()).unwrap(),
            absolute_xp_path: into_absolute_path(DEFAULT_XMRIG_PROXY_PATH.to_string()).unwrap(),
            absolute_node_path: into_absolute_path(DEFAULT_NODE_PATH.to_string()).unwrap(),
            selected_width: APP_DEFAULT_WIDTH as u16,
            selected_height: APP_DEFAULT_HEIGHT as u16,
            selected_scale: APP_DEFAULT_SCALE,
            ratio: Ratio::Width,
            tab: Tab::Xvb,
            show_processes: ProcessName::having_tab(),
        }
    }
}

impl Default for P2pool {
    fn default() -> Self {
        Self {
            simple: true,
            local_node: false,
            mini: true,
            auto_ping: true,
            auto_select: true,
            backup_host: true,
            out_peers: 10,
            in_peers: 10,
            log_level: 3,
            node: RemoteNode::new().to_string(),
            arguments: String::new(),
            address: String::with_capacity(96),
            name: "Local Monero Node".to_string(),
            ip: "localhost".to_string(),
            rpc: "18081".to_string(),
            zmq: "18083".to_string(),
            stratum_port: P2POOL_PORT_DEFAULT,
            selected_node: SelectedPoolNode {
                index: 0,
                name: "Local Monero Node".to_string(),
                ip: "localhost".to_string(),
                rpc: "18081".to_string(),
                zmq_rig: "18083".to_string(),
            },
            prefer_local_node: true,
            console_height: APP_DEFAULT_CONSOLE_HEIGHT,
        }
    }
}

impl Xmrig {
    fn with_threads(max_threads: u16, current_threads: u16) -> Self {
        let xmrig = Self::default();
        Self {
            max_threads,
            current_threads,
            ..xmrig
        }
    }
}
impl Default for Xmrig {
    fn default() -> Self {
        Self {
            simple: true,
            pause: 0,
            simple_rig: String::with_capacity(30),
            arguments: String::with_capacity(300),
            address: String::with_capacity(96),
            name: "Local P2Pool".to_string(),
            rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            ip: "localhost".to_string(),
            port: "3333".to_string(),
            api_ip: "localhost".to_string(),
            api_port: XMRIG_API_PORT_DEFAULT.to_string(),
            tls: false,
            keepalive: false,
            current_threads: 1,
            max_threads: 1,
            selected_pool: SelectedPoolNode {
                index: 0,
                name: "Local Monero Node".to_string(),
                ip: "localhost".to_string(),
                rpc: "18081".to_string(),
                zmq_rig: "18083".to_string(),
            },
            token: rng()
                .sample_iter(Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
            console_height: APP_DEFAULT_CONSOLE_HEIGHT,
        }
    }
}

impl Default for Xvb {
    fn default() -> Self {
        Self {
            simple: true,
            token: String::with_capacity(9),
            simple_hero_mode: Default::default(),
            mode: Default::default(),
            manual_amount_raw: Default::default(),
            manual_slider_amount: Default::default(),
            manual_donation_level: Default::default(),
            manual_donation_metric: Default::default(),
            p2pool_buffer: 25,
            use_p2pool_sidechain_hr: false,
            console_height: APP_DEFAULT_CONSOLE_HEIGHT,
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            gupax: GUPAX_VERSION.to_string(),
            p2pool: P2POOL_VERSION.to_string(),
            xmrig: XMRIG_VERSION.to_string(),
        }
    }
}

// Get the process for the state
impl Node {
    pub const fn process_name() -> ProcessName {
        ProcessName::Node
    }
    pub fn start_options(&self, mode: StartOptionsMode) -> String {
        Helper::build_node_args(self, mode).join(" ")
    }
    /// Return rpc port, zmq port from state
    pub fn ports(&self) -> (u16, u16) {
        let mut zmq_port = NODE_ZMQ_PORT_DEFAULT;
        let mut rpc_port = NODE_RPC_PORT_DEFAULT;
        if self.simple {
            zmq_port = NODE_ZMQ_PORT_DEFAULT;
            rpc_port = NODE_RPC_PORT_DEFAULT;
        } else if !self.arguments.is_empty() {
            // This parses the input and attempts to fill out
            // the [ImgXmrig]... This is pretty bad code...
            let mut last = "";
            for arg in self.arguments.split_whitespace() {
                match last {
                    "--zmq-pub" => {
                        zmq_port = last
                            .split(":")
                            .last()
                            .unwrap_or(&NODE_ZMQ_PORT_DEFAULT.to_string())
                            .parse()
                            .unwrap_or(NODE_ZMQ_PORT_DEFAULT);
                    }
                    "--rpc-bind-port" => zmq_port = last.parse().unwrap_or(NODE_RPC_PORT_DEFAULT),
                    _ => (),
                }
                last = arg;
            }
        } else {
            zmq_port = if self.api_port.is_empty() {
                NODE_ZMQ_PORT_DEFAULT
            } else {
                self.zmq_port.parse().unwrap_or(NODE_ZMQ_PORT_DEFAULT)
            };
            rpc_port = if self.api_port.is_empty() {
                NODE_RPC_PORT_DEFAULT
            } else {
                self.api_port.parse().unwrap_or(NODE_RPC_PORT_DEFAULT)
            };
        }
        (rpc_port, zmq_port)
    }
    /// get the ports that the node process is currently using or that it will use if started with current settings
    pub fn current_ports(&self, alive: bool, img_node: &ImgNode) -> (u16, u16) {
        if alive {
            (img_node.zmq_port, img_node.rpc_port)
        } else {
            self.ports()
        }
    }
}
impl P2pool {
    pub const fn process_name() -> ProcessName {
        ProcessName::P2pool
    }
    pub fn start_options(
        &self,
        path: &Path,
        backup_nodes: &Option<Vec<PoolNode>>,
        mode: StartOptionsMode,
        local_node_zmq_port: u16,
        local_node_rpc_port: u16,
    ) -> String {
        Helper::build_p2pool_args(
            self,
            path,
            backup_nodes,
            false,
            local_node_rpc_port,
            local_node_zmq_port,
            mode,
        )
        .join(" ")
    }
    /// get the port that the p2pool process would use for stratum if it were using the current settings
    pub fn stratum_port(&self) -> u16 {
        if self.simple {
            P2POOL_PORT_DEFAULT
        } else if !self.arguments.is_empty() {
            let mut last = "";
            for arg in self.arguments.split_whitespace() {
                if last == "--stratum" {
                    return last
                        .split(":")
                        .last()
                        .unwrap_or(&P2POOL_PORT_DEFAULT.to_string())
                        .parse()
                        .unwrap_or(P2POOL_PORT_DEFAULT);
                }
                last = arg;
            }
            return P2POOL_PORT_DEFAULT;
        } else {
            return self.stratum_port;
        }
    }

    /// get the ports that the node process is currently using or that it will use if started with current settings
    pub fn current_port(&self, alive: bool, img_p2pool: &ImgP2pool) -> u16 {
        if alive {
            img_p2pool.stratum_port
        } else {
            self.stratum_port()
        }
    }
}
impl Xmrig {
    pub const fn process_name() -> ProcessName {
        ProcessName::Xmrig
    }
    pub fn start_options(&self, mode: StartOptionsMode, p2pool_stratum_port: u16) -> String {
        Helper::build_xmrig_args(self, mode, p2pool_stratum_port).join(" ")
    }
}
impl XmrigProxy {
    pub const fn process_name() -> ProcessName {
        ProcessName::XmrigProxy
    }
    pub fn start_options(&self, mode: StartOptionsMode, p2pool_stratum_port: u16) -> String {
        Helper::build_xp_args(self, mode, p2pool_stratum_port).join(" ")
    }
    /// get the API port that would be used if xmrig was started with the current settings
    pub fn api_port(&self) -> u16 {
        if self.simple {
            PROXY_API_PORT_DEFAULT
        } else if !self.arguments.is_empty() {
            let mut last = "";
            for arg in self.arguments.split_whitespace() {
                if last == "--http-host" {
                    return last.parse().unwrap_or(PROXY_API_PORT_DEFAULT);
                }
                last = arg;
            }
            return PROXY_API_PORT_DEFAULT;
        } else {
            return self.api_port.parse().unwrap_or(PROXY_API_PORT_DEFAULT);
        }
    }
    /// get the port that would be used if xmrig was started with the current settings
    pub fn bind_port(&self) -> u16 {
        if self.simple {
            PROXY_PORT_DEFAULT
        } else if !self.arguments.is_empty() {
            let mut last = "";
            for arg in self.arguments.split_whitespace() {
                if last == "--bind" || last == "-b" {
                    return last
                        .split(":")
                        .last()
                        .unwrap_or_default()
                        .parse()
                        .unwrap_or(PROXY_PORT_DEFAULT);
                }
                last = arg;
            }
            return PROXY_PORT_DEFAULT;
        } else {
            return self.port.parse().unwrap_or(PROXY_PORT_DEFAULT);
        }
    }
    /// get the port that proxy process is currently using or that it will use if started with current settings
    /// return (bind port, api port)
    pub fn current_ports(&self, alive: bool, img_proxy: &ImgProxy) -> (u16, u16) {
        if alive {
            (img_proxy.port, img_proxy.api_port)
        } else {
            (self.bind_port(), self.api_port())
        }
    }
}
// impl Xvb {
//     pub const fn process_name() -> ProcessName {
//         ProcessName::Xvb
//     }
// }

pub enum StartOptionsMode {
    Simple,
    Advanced,
    Custom,
}

impl ProcessName {
    pub fn having_tab() -> Vec<ProcessName> {
        vec![
            ProcessName::Node,
            ProcessName::P2pool,
            ProcessName::Xmrig,
            ProcessName::XmrigProxy,
            ProcessName::Xvb,
        ]
    }
}
