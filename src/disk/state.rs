use anyhow::Result;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use super::*;
use crate::{components::node::RemoteNode, disk::status::*};
//---------------------------------------------------------------------------------------------------- [State] Impl
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let max_threads = benri::threads!();
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
                match read_to_string(file, path) {
                    Ok(s) => s,
                    Err(e) => return Err(e),
                }
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
    pub auto_update: bool,
    pub auto_p2pool: bool,
    pub auto_node: bool,
    pub auto_xmrig: bool,
    pub auto_xp: bool,
    pub auto_xvb: bool,
    pub ask_before_quit: bool,
    pub save_before_quit: bool,
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
    pub bundled: bool,
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
    pub log_level: u8,
    pub node: String,
    pub arguments: String,
    pub address: String,
    pub name: String,
    pub ip: String,
    pub rpc: String,
    pub zmq: String,
    pub selected_index: usize,
    pub selected_name: String,
    pub selected_ip: String,
    pub selected_rpc: String,
    pub selected_zmq: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Node {
    pub simple: bool,
    pub api_ip: String,
    pub api_port: String,
    pub out_peers: u16,
    pub in_peers: u16,
    pub log_level: u8,
    pub arguments: String,
    pub zmq_ip: String,
    pub zmq_port: String,
    pub pruned: bool,
    pub dns_blocklist: bool,
    pub disable_dns_checkpoint: bool,
    pub path_db: String,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            simple: true,
            api_ip: String::from("127.0.0.1"),
            api_port: 18081.to_string(),
            out_peers: 32,
            in_peers: 64,
            log_level: 0,
            arguments: String::new(),
            zmq_ip: String::from("127.0.0.1"),
            zmq_port: 18083.to_string(),
            pruned: true,
            dns_blocklist: true,
            disable_dns_checkpoint: true,
            path_db: String::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Xmrig {
    pub simple: bool,
    pub pause: u8,
    pub simple_rig: String,
    pub arguments: String,
    pub tls: bool,
    pub keepalive: bool,
    pub max_threads: usize,
    pub current_threads: usize,
    pub address: String,
    pub api_ip: String,
    pub api_port: String,
    pub name: String,
    pub rig: String,
    pub ip: String,
    pub port: String,
    pub selected_index: usize,
    pub selected_name: String,
    pub selected_rig: String,
    pub selected_ip: String,
    pub selected_port: String,
    pub token: String,
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
    pub selected_index: usize,
    pub selected_name: String,
    pub selected_rig: String,
    pub selected_ip: String,
    pub selected_port: String,
    pub token: String,
    pub redirect_local_xmrig: bool,
}

impl Default for XmrigProxy {
    fn default() -> Self {
        XmrigProxy {
            simple: true,
            arguments: Default::default(),
            token: thread_rng()
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
            selected_index: 0,
            selected_name: "Local P2Pool".to_string(),
            selected_ip: "localhost".to_string(),
            selected_rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            selected_port: "3333".to_string(),
            api_ip: "localhost".to_string(),
            api_port: "18089".to_string(),
            tls: false,
            keepalive: false,
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
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, Default)]
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
            auto_update: false,
            auto_p2pool: false,
            auto_node: false,
            auto_xmrig: false,
            auto_xp: false,
            auto_xvb: false,
            ask_before_quit: true,
            save_before_quit: true,
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
            #[cfg(feature = "bundle")]
            bundled: true,
            #[cfg(not(feature = "bundle"))]
            bundled: false,
        }
    }
}

impl Default for P2pool {
    fn default() -> Self {
        Self {
            simple: true,
            local_node: true,
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
            selected_index: 0,
            selected_name: "Local Monero Node".to_string(),
            selected_ip: "localhost".to_string(),
            selected_rpc: "18081".to_string(),
            selected_zmq: "18083".to_string(),
        }
    }
}

impl Xmrig {
    fn with_threads(max_threads: usize, current_threads: usize) -> Self {
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
            selected_index: 0,
            selected_name: "Local P2Pool".to_string(),
            selected_ip: "localhost".to_string(),
            selected_rig: GUPAX_VERSION_UNDERSCORE.to_string(),
            selected_port: "3333".to_string(),
            api_ip: "localhost".to_string(),
            api_port: "18088".to_string(),
            tls: false,
            keepalive: false,
            current_threads: 1,
            max_threads: 1,
            token: thread_rng()
                .sample_iter(Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
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
            p2pool_buffer: 5,
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
