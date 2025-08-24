use std::fmt::Display;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::utils::constants::{
    P2POOL_ADVANCED, P2POOL_CRAWLER, P2POOL_SIMPLE, STATUS_SUBMENU_HASHRATE, STATUS_SUBMENU_P2POOL,
    STATUS_SUBMENU_PROCESSES,
};

/// A submenu
pub trait Submenu: Display + PartialEq + Default + IntoEnumIterator {
    fn hover_text(&self) -> &'static str;
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize, Default, EnumIter)]
pub enum SubmenuStatus {
    #[default]
    Processes,
    P2pool,
    Benchmarks,
}

impl Display for SubmenuStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SubmenuStatus::P2pool => write!(f, "P2Pool"),
            _ => write!(f, "{self:?}"),
        }
    }
}
impl Submenu for SubmenuStatus {
    fn hover_text(&self) -> &'static str {
        match self {
            Self::Processes => STATUS_SUBMENU_PROCESSES,
            Self::P2pool => STATUS_SUBMENU_P2POOL,
            Self::Benchmarks => STATUS_SUBMENU_HASHRATE,
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize, Default, Display, EnumIter)]
pub enum SubmenuP2pool {
    #[default]
    Simple,
    Advanced,
    Crawler,
}

impl Submenu for SubmenuP2pool {
    fn hover_text(&self) -> &'static str {
        match self {
            Self::Simple => P2POOL_SIMPLE,
            Self::Advanced => P2POOL_ADVANCED,
            Self::Crawler => P2POOL_CRAWLER,
        }
    }
}
