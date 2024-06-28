use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::helper::xvb::api_url_xmrig;
use crate::helper::xvb::current_controllable_hr;
use crate::helper::ProcessName;
use crate::miscs::output_console;
use crate::miscs::output_console_without_time;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, info, warn};
use readable::num::Float;
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    helper::{
        p2pool::PubP2poolApi,
        xrig::{update_xmrig_config, xmrig::PubXmrigApi},
        xvb::nodes::XvbNode,
    },
    macros::lock,
    BLOCK_PPLNS_WINDOW_MAIN, BLOCK_PPLNS_WINDOW_MINI, SECOND_PER_BLOCK_P2POOL, XVB_BUFFER,
    XVB_ROUND_DONOR_MEGA_MIN_HR, XVB_ROUND_DONOR_MIN_HR, XVB_ROUND_DONOR_VIP_MIN_HR,
    XVB_ROUND_DONOR_WHALE_MIN_HR, XVB_TIME_ALGO,
};

use super::{PubXvbApi, SamplesAverageHour};

pub(crate) fn calcul_donated_time(
    lhr: f32,
    gui_api_p2pool: &Arc<Mutex<PubP2poolApi>>,
    gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
    state_p2pool: &crate::disk::state::P2pool,
) -> u32 {
    let p2pool_ehr = lock!(gui_api_p2pool).sidechain_ehr;
    // what if ehr stay still for the next ten minutes ? mHR will augment every ten minutes because it thinks that oHR is decreasing.
    //
    let avg_hr = calc_last_hour_avg_hash_rate(&lock!(gui_api_xvb).p2pool_sent_last_hour_samples);
    let mut p2pool_ohr = p2pool_ehr - avg_hr;
    if p2pool_ohr < 0.0 {
        p2pool_ohr = 0.0;
    }
    info!("XvB Process | p2pool sidechain HR - last hour average HR = estimated outside HR\n{p2pool_ehr} - {avg_hr} = {p2pool_ohr}");
    let mut min_hr = minimum_hashrate_share(
        lock!(gui_api_p2pool).p2pool_difficulty_u64,
        state_p2pool.mini,
        p2pool_ohr,
    );
    if min_hr.is_sign_negative() {
        info!("XvB Process | if minimum HR is negative, it is 0.");
        min_hr = 0.0;
    }
    info!("Xvb Process | hr {}, min_hr: {} ", lhr, min_hr);
    // numbers are divided by a thousands to print kH/s and not H/s
    let msg_lhr = format!(
        "{} kH/s local HR from Xmrig",
        Float::from_3((lhr / 1000.0).into())
    );
    let msg_mhr = format!(
        "{} kH/s minimum required local HR to keep a share in PPLNS window",
        Float::from_3((min_hr / 1000.0).into())
    );
    let msg_ehr = format!(
        "{} kH/s estimated sent the last hour for your address on p2pool, including this instance",
        Float::from_3((p2pool_ehr / 1000.0).into())
    );
    output_console(
        &mut lock!(gui_api_xvb).output,
        &msg_lhr,
        crate::helper::ProcessName::Xvb,
    );
    output_console(
        &mut lock!(gui_api_xvb).output,
        &msg_mhr,
        crate::helper::ProcessName::Xvb,
    );
    output_console(
        &mut lock!(gui_api_xvb).output,
        &msg_ehr,
        crate::helper::ProcessName::Xvb,
    );
    // calculate how much time can be spared
    let mut spared_time = time_that_could_be_spared(lhr, min_hr);

    if spared_time > 0 {
        // if not hero option
        if !lock!(gui_api_xvb).stats_priv.runtime_hero_mode {
            let xvb_chr = lock!(gui_api_xvb).stats_priv.donor_1hr_avg * 1000.0;
            info!("current HR on XvB (last hour): {xvb_chr}");
            let shr = calc_last_hour_avg_hash_rate(&lock!(gui_api_xvb).xvb_sent_last_hour_samples);
            // calculate how much time needed to be spared to be in most round type minimum HR + buffer
            spared_time = minimum_time_for_highest_accessible_round(spared_time, lhr, xvb_chr, shr);
        }
    }
    if lock!(gui_api_xvb).stats_priv.runtime_hero_mode {
        output_console(
            &mut lock!(gui_api_xvb).output,
            "Hero mode is enabled for this decision",
            crate::helper::ProcessName::Xvb,
        );
    }
    spared_time
}
fn minimum_hashrate_share(difficulty: u64, mini: bool, ohr: f32) -> f32 {
    let pws = if mini {
        BLOCK_PPLNS_WINDOW_MINI
    } else {
        BLOCK_PPLNS_WINDOW_MAIN
    };
    let minimum_hr = ((difficulty / (pws * SECOND_PER_BLOCK_P2POOL)) as f32 * XVB_BUFFER) - ohr;
    info!("XvB Process | (difficulty / (window pplns blocks * seconds per p2pool block) * BUFFER) - outside HR = minimum HR to keep a share\n({difficulty} / ({pws} * {SECOND_PER_BLOCK_P2POOL}) * {XVB_BUFFER}) - {ohr} = {minimum_hr}");
    minimum_hr
}
fn time_that_could_be_spared(hr: f32, min_hr: f32) -> u32 {
    // percent of time minimum
    let minimum_time_required_on_p2pool = XVB_TIME_ALGO as f32 / (hr / min_hr);
    info!("XvB Process | Time of algo / local hashrate / minimum hashrate = minimum time required on p2pool\n{XVB_TIME_ALGO} / ({hr} / {min_hr}) = {minimum_time_required_on_p2pool}");
    let spared_time = XVB_TIME_ALGO as f32 - minimum_time_required_on_p2pool;
    info!("XvB Process | Time of algo - minimum time required on p2pool = time that can be spared.\n{XVB_TIME_ALGO} - {minimum_time_required_on_p2pool} = {spared_time}");
    // if less than 6 seconds, XMRig could hardly have the time to mine anything.
    if spared_time >= 6.0 {
        return spared_time as u32;
    }
    info!(
        "XvB Process | sparted time is equal or less than 6 seconds, so everything goes to p2pool."
    );
    0
}

// spared time, local hr, current 1h average hr already mining on XvB, 1h average local HR sent on XvB.
fn minimum_time_for_highest_accessible_round(st: u32, lhr: f32, chr: f32, shr: f32) -> u32 {
    // we remove one second that could possibly be sent, because if the time needed is a float, it will be rounded up.
    // this subtraction can not fail because mnimum spared time is >= 6.
    let hr_for_xvb = ((st - 1) as f32 / XVB_TIME_ALGO as f32) * lhr;
    info!(
        "hr for xvb is: ({st} / {}) * {lhr} = {hr_for_xvb}H/s",
        XVB_TIME_ALGO
    );
    let ohr = chr - shr;
    info!("ohr is: {chr} - {shr} = {ohr}H/s");
    let min_mega = XVB_ROUND_DONOR_MEGA_MIN_HR as f32 - ohr;
    info!(
        "minimum required HR for mega round is: {} - {ohr} = {min_mega}H/s",
        XVB_ROUND_DONOR_MEGA_MIN_HR
    );
    let min_whale = XVB_ROUND_DONOR_WHALE_MIN_HR as f32 - ohr;
    info!(
        "minimum required HR for whale round is: {} - {ohr} = {min_whale}H/s",
        XVB_ROUND_DONOR_WHALE_MIN_HR
    );
    let min_donorvip = XVB_ROUND_DONOR_VIP_MIN_HR as f32 - ohr;
    info!(
        "minimum required HR for donor vip round is: {} - {ohr} = {min_donorvip}H/s",
        XVB_ROUND_DONOR_VIP_MIN_HR
    );
    let min_donor = XVB_ROUND_DONOR_MIN_HR as f32 - ohr;
    info!(
        "minimum required HR for donor round is: {} - {ohr} = {min_donor}H/s",
        XVB_ROUND_DONOR_MIN_HR
    );
    let min = match hr_for_xvb {
        x if x > min_mega => {
            info!("trying to get Mega round");
            info!(
                "minimum second to send = ((({x} - ({x} - {min_mega})) / {lhr}) * {}) ",
                XVB_TIME_ALGO
            );
            min_mega
        }
        x if x > min_whale => {
            info!("trying to get Whale round");
            info!(
                "minimum second to send = ((({x} - ({x} - {min_whale})) / {lhr}) * {}) ",
                XVB_TIME_ALGO
            );
            min_whale
        }
        x if x > min_donorvip => {
            info!("trying to get Vip Donor round");
            info!(
                "minimum second to send = ((({x} - ({x} - {min_donorvip})) / {lhr}) * {}) ",
                XVB_TIME_ALGO
            );
            min_donorvip
        }
        x if x > min_donor => {
            info!("trying to get Donor round");
            info!(
                "minimum second to send = ((({x} - ({x} - {min_donor})) / {lhr}) * {}) ",
                XVB_TIME_ALGO
            );
            min_donor
        }
        _ => return 0,
    };

    (((hr_for_xvb - (hr_for_xvb - min)) / lhr) * XVB_TIME_ALGO as f32).ceil() as u32
}
#[allow(clippy::too_many_arguments)]
async fn sleep_then_update_node_xmrig(
    spared_time: u32,
    client: &Client,
    api_uri: &str,
    token_xmrig: &str,
    address: &str,
    gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
    gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
    gui_api_xp: &Arc<Mutex<PubXmrigProxyApi>>,
    rig: &str,
    xp_alive: bool,
) {
    let node = lock!(gui_api_xvb).stats_priv.node;
    debug!(
        "Xvb Process | algo sleep for {} while mining on P2pool",
        XVB_TIME_ALGO - spared_time
    );
    let msg_xmrig_or_xp = if xp_alive { "XMRig-Proxy" } else { "XMRig" };
    sleep(Duration::from_secs((XVB_TIME_ALGO - spared_time).into())).await;
    // only update xmrig config if it is actually mining.
    if spared_time > 0 {
        debug!("Xvb Process | request {msg_xmrig_or_xp} to mine on XvB");
        if lock!(gui_api_xvb).current_node.is_none()
            || lock!(gui_api_xvb)
                .current_node
                .as_ref()
                .is_some_and(|n| n == &XvbNode::P2pool)
        {
            if let Err(err) =
                update_xmrig_config(client, api_uri, token_xmrig, &node, address, rig).await
            {
                // show to console error about updating xmrig config
                warn!("Xvb Process | Failed request HTTP API {msg_xmrig_or_xp}");
                output_console(
                    &mut lock!(gui_api_xvb).output,
                    &format!(
                        "Failure to update {msg_xmrig_or_xp} config with HTTP API.\nError: {}",
                        err
                    ),
                    crate::helper::ProcessName::Xvb,
                );
            } else {
                if xp_alive {
                    lock!(gui_api_xp).node = node.to_string();
                } else {
                    lock!(gui_api_xmrig).node = node.to_string();
                }
                debug!("Xvb Process | mining on XvB pool");
            }
        }
        // will not quit the process until it is really done.
        // xvb process watch this algo handle to see if process is finished or not.
        sleep(Duration::from_secs(spared_time.into())).await;
    }
}
// push new value into samples before executing this calcul
fn calc_last_hour_avg_hash_rate(samples: &SamplesAverageHour) -> f32 {
    samples.0.iter().sum::<f32>() / samples.0.len() as f32
}
#[allow(clippy::too_many_arguments)]
pub(crate) async fn algorithm(
    client: &Client,
    gui_api_xvb: &Arc<Mutex<PubXvbApi>>,
    gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
    gui_api_xp: &Arc<Mutex<PubXmrigProxyApi>>,
    gui_api_p2pool: &Arc<Mutex<PubP2poolApi>>,
    token_xmrig: &str,
    state_p2pool: &crate::disk::state::P2pool,
    share: u32,
    time_donated: &Arc<Mutex<u32>>,
    rig: &str,
    xp_alive: bool,
) {
    debug!("Xvb Process | Algorithm is started");
    output_console(
        &mut lock!(gui_api_xvb).output,
        "Algorithm of distribution HR started for the next ten minutes.",
        ProcessName::Xvb,
    );
    // the time that takes the algorithm do decide the next ten minutes could means less p2pool mining. It is solved by the buffer and spawning requests.
    let address = &state_p2pool.address;
    // request XMrig to mine on P2pool
    // if share is in PW,
    let msg_xmrig_or_xp = if xp_alive { "XMRig-Proxy" } else { "XMRig" };

    let api_url = api_url_xmrig(xp_alive, true);
    if share > 0 {
        debug!("Xvb Process | Algorithm share is in current window");
        // calcul minimum HR

        output_console(
            &mut lock!(gui_api_xvb).output,
            "At least one share is in current PPLNS window.",
            ProcessName::Xvb,
        );
        let hashrate_xmrig = current_controllable_hr(xp_alive, gui_api_xp, gui_api_xmrig);
        *lock!(time_donated) =
            calcul_donated_time(hashrate_xmrig, gui_api_p2pool, gui_api_xvb, state_p2pool);
        let time_donated = *lock!(time_donated);
        debug!("Xvb Process | Donated time {} ", time_donated);
        output_console(
            &mut lock!(gui_api_xvb).output,
            &format!(
                "Mining on P2pool node for {} seconds then on XvB for {} seconds.",
                XVB_TIME_ALGO - time_donated,
                time_donated
            ),
            ProcessName::Xvb,
        );

        // p2pool need to be mined if donated time is not equal to xvb_time_algo
        if time_donated != XVB_TIME_ALGO && lock!(gui_api_xvb).current_node != Some(XvbNode::P2pool)
        {
            debug!("Xvb Process | request {msg_xmrig_or_xp} to mine on p2pool");
            if let Err(err) = update_xmrig_config(
                client,
                &api_url,
                token_xmrig,
                &XvbNode::P2pool,
                address,
                rig,
            )
            .await
            {
                warn!("Xvb Process | Failed request HTTP API {msg_xmrig_or_xp}");
                output_console(
                    &mut lock!(gui_api_xvb).output,
                    &format!(
                        "Failure to update {msg_xmrig_or_xp} config with HTTP API.\nError: {}",
                        err
                    ),
                    ProcessName::Xvb,
                );
            } else if xp_alive {
                lock!(gui_api_xmrig).node = XvbNode::P2pool.to_string()
            } else {
                lock!(gui_api_xmrig).node = XvbNode::P2pool.to_string();
            }
        }

        // sleep 10m less spared time then request XMrig to mine on XvB
        sleep_then_update_node_xmrig(
            time_donated,
            client,
            &api_url,
            token_xmrig,
            address,
            gui_api_xvb,
            gui_api_xmrig,
            gui_api_xp,
            "",
            xp_alive,
        )
        .await;
        lock!(gui_api_xvb)
            .p2pool_sent_last_hour_samples
            .0
            .push_back(
                hashrate_xmrig
                    * ((XVB_TIME_ALGO as f32 - time_donated as f32) / XVB_TIME_ALGO as f32),
            );
        lock!(gui_api_xvb)
            .xvb_sent_last_hour_samples
            .0
            .push_back(hashrate_xmrig * (time_donated as f32 / XVB_TIME_ALGO as f32));
    } else {
        // no share, so we mine on p2pool. We update xmrig only if it was still mining on XvB.
        if lock!(gui_api_xvb).current_node != Some(XvbNode::P2pool) {
            info!("Xvb Process | request {msg_xmrig_or_xp}to mine on p2pool");

            if let Err(err) = update_xmrig_config(
                client,
                &api_url,
                token_xmrig,
                &XvbNode::P2pool,
                address,
                rig,
            )
            .await
            {
                warn!("Xvb Process | Failed request HTTP API {msg_xmrig_or_xp}");
                output_console(
                    &mut lock!(gui_api_xvb).output,
                    &format!(
                        "Failure to update {msg_xmrig_or_xp}config with HTTP API.\nError: {}",
                        err
                    ),
                    ProcessName::Xvb,
                );
            }
        }
        output_console(
            &mut lock!(gui_api_xvb).output,
            "No share in the current PPLNS Window !",
            ProcessName::Xvb,
        );
        output_console(
            &mut lock!(gui_api_xvb).output,
            "Mining on P2pool for the next ten minutes.",
            ProcessName::Xvb,
        );
        sleep(Duration::from_secs(XVB_TIME_ALGO.into())).await;
        let hr = current_controllable_hr(xp_alive, gui_api_xp, gui_api_xmrig);
        lock!(gui_api_xvb)
            .p2pool_sent_last_hour_samples
            .0
            .push_back(hr);
        lock!(gui_api_xvb)
            .xvb_sent_last_hour_samples
            .0
            .push_back(0.0);
    }
    // algorithm has run, so do not retry but run normally
    // put a space to mark the difference with the next run.
    output_console_without_time(&mut lock!(gui_api_xvb).output, "", ProcessName::Xvb);
}
