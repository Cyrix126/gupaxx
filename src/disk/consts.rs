//---------------------------------------------------------------------------------------------------- Const
// State file
pub const ERROR: &str = "Disk error";
pub const PATH_ERROR: &str = "PATH for state directory could not be not found";

#[cfg(target_os = "windows")]
pub const DIRECTORY: &str = r#"Gupaxx\"#;
#[cfg(target_os = "macos")]
pub const DIRECTORY: &str = "Gupaxx/";
#[cfg(target_os = "linux")]
pub const DIRECTORY: &str = "gupaxx/";

// File names
pub const STATE_TOML: &str = "state.toml";
pub const NODE_TOML: &str = "node.toml";
pub const POOL_TOML: &str = "pool.toml";

// P2Pool API
// Lives within the Gupax OS data directory.
// ~/.local/share/gupax/p2pool/
// ├─ payout_log  // Raw log lines of payouts received
// ├─ payout      // Single [u64] representing total payouts
// ├─ xmr         // Single [u64] representing total XMR mined in atomic units
#[cfg(target_os = "windows")]
pub const GUPAX_P2POOL_API_DIRECTORY: &str = r"p2pool\";
#[cfg(target_family = "unix")]
pub const GUPAX_P2POOL_API_DIRECTORY: &str = "p2pool/";
pub const GUPAX_P2POOL_API_LOG: &str = "log";
pub const GUPAX_P2POOL_API_PAYOUT: &str = "payout";
pub const GUPAX_P2POOL_API_XMR: &str = "xmr";
pub const GUPAX_P2POOL_API_FILE_ARRAY: [&str; 3] = [
    GUPAX_P2POOL_API_LOG,
    GUPAX_P2POOL_API_PAYOUT,
    GUPAX_P2POOL_API_XMR,
];

#[cfg(target_os = "windows")]
pub const DEFAULT_P2POOL_PATH: &str = r"P2Pool\p2pool.exe";
#[cfg(target_os = "macos")]
pub const DEFAULT_P2POOL_PATH: &str = "p2pool/p2pool";
#[cfg(target_os = "windows")]
pub const DEFAULT_XMRIG_PATH: &str = r"XMRig\xmrig.exe";
#[cfg(target_os = "windows")]
pub const DEFAULT_XMRIG_PROXY_PATH: &str = r"XMRig-Proxy\xmrig-proxy.exe";
#[cfg(target_os = "macos")]
pub const DEFAULT_XMRIG_PATH: &str = "xmrig/xmrig";
#[cfg(target_os = "macos")]
pub const DEFAULT_XMRIG_PROXY_PATH: &str = "xmrig-proxy/xmrig-proxy";

// Default to [/usr/bin/] for Linux distro builds.
#[cfg(target_os = "linux")]
#[cfg(not(feature = "distro"))]
pub const DEFAULT_P2POOL_PATH: &str = "p2pool/p2pool";
#[cfg(target_os = "linux")]
#[cfg(not(feature = "distro"))]
pub const DEFAULT_XMRIG_PATH: &str = "xmrig/xmrig";
#[cfg(target_os = "linux")]
#[cfg(not(feature = "distro"))]
pub const DEFAULT_XMRIG_PROXY_PATH: &str = "xmrig-proxy/xmrig-proxy";
#[cfg(target_os = "linux")]
#[cfg(feature = "distro")]
pub const DEFAULT_P2POOL_PATH: &str = "/usr/bin/p2pool";
#[cfg(target_os = "linux")]
#[cfg(feature = "distro")]
pub const DEFAULT_XMRIG_PATH: &str = "/usr/bin/xmrig";
#[cfg(target_os = "linux")]
#[cfg(feature = "distro")]
pub const DEFAULT_XMRIG_PROXY_PATH: &str = "/usr/bin/xmrig-proxy";
