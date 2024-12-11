use derive_more::derive::Display;
use strum::{EnumCount, EnumIter};

use super::*;
//---------------------------------------------------------------------------------------------------- [Submenu] enum for [Status] tab
#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum Submenu {
    Processes,
    P2pool,
    Benchmarks,
}

impl Default for Submenu {
    fn default() -> Self {
        Self::Processes
    }
}

impl Display for Submenu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Submenu::*;
        match self {
            P2pool => write!(f, "P2Pool"),
            _ => write!(f, "{:?}", self),
        }
    }
}

//---------------------------------------------------------------------------------------------------- [PayoutView] enum for [Status/P2Pool] tab
// The enum buttons for selecting which "view" to sort the payout log in.
#[derive(
    Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize, Display, EnumIter, EnumCount,
)]
pub enum PayoutView {
    Latest,   // Shows the most recent logs first
    Oldest,   // Shows the oldest logs first
    Biggest,  // Shows highest to lowest payouts
    Smallest, // Shows lowest to highest payouts
}

impl PayoutView {
    pub const fn msg_help(&self) -> &str {
        match self {
            Self::Latest => STATUS_SUBMENU_LATEST,
            Self::Oldest => STATUS_SUBMENU_OLDEST,
            Self::Biggest => STATUS_SUBMENU_SMALLEST,
            Self::Smallest => STATUS_SUBMENU_BIGGEST,
        }
    }
}

impl PayoutView {
    fn new() -> Self {
        Self::Latest
    }
}

impl Default for PayoutView {
    fn default() -> Self {
        Self::new()
    }
}

//---------------------------------------------------------------------------------------------------- [Hash] enum for [Status/P2Pool]
#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[allow(clippy::enum_variant_names)]
pub enum Hash {
    Hash,
    Kilo,
    Mega,
    Giga,
}

impl Default for Hash {
    fn default() -> Self {
        Self::Hash
    }
}

impl Hash {
    pub fn convert_to_hash(f: f64, from: Self) -> f64 {
        match from {
            Self::Hash => f,
            Self::Kilo => f * 1_000.0,
            Self::Mega => f * 1_000_000.0,
            Self::Giga => f * 1_000_000_000.0,
        }
    }
    #[cfg(test)]
    pub fn convert(f: f64, og: Self, new: Self) -> f64 {
        match og {
            Self::Hash => match new {
                Self::Hash => f,
                Self::Kilo => f / 1_000.0,
                Self::Mega => f / 1_000_000.0,
                Self::Giga => f / 1_000_000_000.0,
            },
            Self::Kilo => match new {
                Self::Hash => f * 1_000.0,
                Self::Kilo => f,
                Self::Mega => f / 1_000.0,
                Self::Giga => f / 1_000_000.0,
            },
            Self::Mega => match new {
                Self::Hash => f * 1_000_000.0,
                Self::Kilo => f * 1_000.0,
                Self::Mega => f,
                Self::Giga => f / 1_000.0,
            },
            Self::Giga => match new {
                Self::Hash => f * 1_000_000_000.0,
                Self::Kilo => f * 1_000_000.0,
                Self::Mega => f * 1_000.0,
                Self::Giga => f,
            },
        }
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Hash::Hash => write!(f, "H/s"),
            Hash::Kilo => write!(f, "KH/s"),
            Hash::Mega => write!(f, "MH/s"),
            Hash::Giga => write!(f, "GH/s"),
        }
    }
}
