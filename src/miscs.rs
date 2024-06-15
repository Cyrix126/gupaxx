//---------------------------------------------------------------------------------------------------- Misc functions

// Get absolute [Gupax] binary path
#[cold]
#[inline(never)]
pub fn get_exe() -> Result<String, std::io::Error> {
    match std::env::current_exe() {
        Ok(path) => Ok(path.display().to_string()),
        Err(err) => {
            error!("Couldn't get absolute Gupax PATH");
            Err(err)
        }
    }
}

// Get absolute [Gupax] directory path
#[cold]
#[inline(never)]
pub fn get_exe_dir() -> Result<String, std::io::Error> {
    match std::env::current_exe() {
        Ok(mut path) => {
            path.pop();
            Ok(path.display().to_string())
        }
        Err(err) => {
            error!("Couldn't get exe basepath PATH");
            Err(err)
        }
    }
}

// Clean any [gupax_update_.*] directories
// The trailing random bits must be exactly 10 alphanumeric characters
#[cold]
#[inline(never)]
pub fn clean_dir() -> Result<(), anyhow::Error> {
    let regex = Regex::new("^gupaxx_update_[A-Za-z0-9]{10}$").unwrap();
    for entry in std::fs::read_dir(get_exe_dir()?)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        if Regex::is_match(
            &regex,
            entry
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow::Error::msg("Basename failed"))?,
        ) {
            let path = entry.path();
            match std::fs::remove_dir_all(&path) {
                Ok(_) => info!("Remove [{}] ... OK", path.display()),
                Err(e) => warn!("Remove [{}] ... FAIL ... {}", path.display(), e),
            }
        }
    }
    Ok(())
}

// Print disk files to console
#[cold]
#[inline(never)]
pub fn print_disk_file(path: &PathBuf) {
    match std::fs::read_to_string(path) {
        Ok(string) => {
            print!("{}", string);
            exit(0);
        }
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    }
}

// Prints the GupaxP2PoolApi files.
#[cold]
#[inline(never)]
pub fn print_gupax_p2pool_api(gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>) {
    let api = lock!(gupax_p2pool_api);
    let log = match std::fs::read_to_string(&api.path_log) {
        Ok(string) => string,
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    };
    let payout = match std::fs::read_to_string(&api.path_payout) {
        Ok(string) => string,
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    };
    let xmr = match std::fs::read_to_string(&api.path_xmr) {
        Ok(string) => string,
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    };
    let xmr = match xmr.trim().parse::<u64>() {
        Ok(o) => crate::xmr::AtomicUnit::from_u64(o),
        Err(e) => {
            warn!("GupaxP2poolApi | [xmr] parse error: {}", e);
            exit(1);
        }
    };
    println!(
        "{}\nTotal payouts | {}\nTotal XMR     | {} ({} Atomic Units)",
        log,
        payout.trim(),
        xmr,
        xmr.to_u64()
    );
    exit(0);
}

#[inline]
pub fn cmp_f64(a: f64, b: f64) -> std::cmp::Ordering {
    match (a <= b, a >= b) {
        (false, true) => std::cmp::Ordering::Greater,
        (true, false) => std::cmp::Ordering::Less,
        (true, true) => std::cmp::Ordering::Equal,
        _ => std::cmp::Ordering::Less,
    }
}
// Free functions.

use crate::disk::gupax_p2pool_api::GupaxP2poolApi;
use crate::utils::macros::lock;
use log::error;
use log::warn;
use regex::Regex;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::sync::Mutex;

use log::info;

//---------------------------------------------------------------------------------------------------- Use
use crate::constants::*;

//----------------------------------------------------------------------------------------------------
#[cold]
#[inline(never)]
// Clamp the scaling resolution `f32` to a known good `f32`.
pub fn clamp_scale(scale: f32) -> f32 {
    // Make sure it is finite.
    if !scale.is_finite() {
        return APP_DEFAULT_SCALE;
    }

    // Clamp between valid range.
    scale.clamp(APP_MIN_SCALE, APP_MAX_SCALE)
}
