use std::sync::{Arc, Mutex};

use derive_more::Display;
use serde::Deserialize;

use crate::{
    macros::lock, XVB_ROUND_DONOR_MEGA_MIN_HR, XVB_ROUND_DONOR_MIN_HR, XVB_ROUND_DONOR_VIP_MIN_HR,
    XVB_ROUND_DONOR_WHALE_MIN_HR,
};

use super::PubXvbApi;
#[derive(Debug, Clone, Default, Display, Deserialize, PartialEq)]
pub enum XvbRound {
    #[default]
    #[display(fmt = "VIP")]
    #[serde(alias = "vip")]
    Vip,
    #[serde(alias = "donor")]
    Donor,
    #[display(fmt = "VIP Donor")]
    #[serde(alias = "donor_vip")]
    DonorVip,
    #[display(fmt = "Whale Donor")]
    #[serde(alias = "donor_whale")]
    DonorWhale,
    #[display(fmt = "Mega Donor")]
    #[serde(alias = "donor_mega")]
    DonorMega,
}

pub(crate) fn round_type(share: u32, pub_api: &Arc<Mutex<PubXvbApi>>) -> Option<XvbRound> {
    if share > 0 {
        let stats_priv = &lock!(pub_api).stats_priv;
        match (
            (stats_priv.donor_1hr_avg * 1000.0) as u32,
            (stats_priv.donor_24hr_avg * 1000.0) as u32,
        ) {
            x if x.0 >= XVB_ROUND_DONOR_MEGA_MIN_HR && x.1 >= XVB_ROUND_DONOR_MEGA_MIN_HR => {
                Some(XvbRound::DonorMega)
            }
            x if x.0 >= XVB_ROUND_DONOR_WHALE_MIN_HR && x.1 >= XVB_ROUND_DONOR_WHALE_MIN_HR => {
                Some(XvbRound::DonorWhale)
            }
            x if x.0 >= XVB_ROUND_DONOR_VIP_MIN_HR && x.1 >= XVB_ROUND_DONOR_VIP_MIN_HR => {
                Some(XvbRound::DonorVip)
            }
            x if x.0 >= XVB_ROUND_DONOR_MIN_HR && x.1 >= XVB_ROUND_DONOR_MIN_HR => {
                Some(XvbRound::Donor)
            }
            (_, _) => Some(XvbRound::Vip),
        }
    } else {
        None
    }
}
