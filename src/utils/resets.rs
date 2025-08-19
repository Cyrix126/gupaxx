//---------------------------------------------------------------------------------------------------- Reset functions
use crate::disk::create_gupax_dir;
use crate::disk::errors::TomlError;
use crate::disk::gupax_p2pool_api::GupaxP2poolApi;
use crate::disk::node::Node;
use crate::disk::pool::Pool;
use crate::disk::state::State;
use crate::info;
use log::error;
use std::path::PathBuf;
use std::process::exit;

#[cold]
#[inline(never)]
pub fn reset_state(path: &PathBuf) -> Result<(), TomlError> {
    match State::create_new(path) {
        Ok(_) => {
            info!("Resetting [state.toml] ... OK");
            Ok(())
        }
        Err(e) => {
            error!("Resetting [state.toml] ... FAIL ... {e}");
            Err(e)
        }
    }
}

#[cold]
#[inline(never)]
pub fn reset_nodes(path: &PathBuf) -> Result<(), TomlError> {
    match Node::create_new(path) {
        Ok(_) => {
            info!("Resetting [node.toml] ... OK");
            Ok(())
        }
        Err(e) => {
            error!("Resetting [node.toml] ... FAIL ... {e}");
            Err(e)
        }
    }
}

#[cold]
#[inline(never)]
pub fn reset_pools(path: &PathBuf) -> Result<(), TomlError> {
    match Pool::create_new(path) {
        Ok(_) => {
            info!("Resetting [pool.toml] ... OK");
            Ok(())
        }
        Err(e) => {
            error!("Resetting [pool.toml] ... FAIL ... {e}");
            Err(e)
        }
    }
}

#[cold]
#[inline(never)]
pub fn reset_gupax_p2pool_api(path: &PathBuf) -> Result<(), TomlError> {
    match GupaxP2poolApi::create_new(path) {
        Ok(_) => {
            info!("Resetting GupaxP2poolApi ... OK");
            Ok(())
        }
        Err(e) => {
            error!("Resetting GupaxP2poolApi folder ... FAIL ... {e}");
            Err(e)
        }
    }
}

#[cold]
#[inline(never)]
pub fn reset(
    path: &PathBuf,
    state: &PathBuf,
    node: &PathBuf,
    pool: &PathBuf,
    gupax_p2pool_api: &PathBuf,
) {
    let mut code = 0;
    // Attempt to remove directory first
    match std::fs::remove_dir_all(path) {
        Ok(_) => info!("Removing OS data path ... OK"),
        Err(e) => {
            error!("Removing OS data path ... FAIL ... {e}");
            code = 1;
        }
    }
    // Recreate
    match create_gupax_dir(path) {
        Ok(_) => (),
        Err(_) => code = 1,
    }
    match reset_state(state) {
        Ok(_) => (),
        Err(_) => code = 1,
    }
    match reset_nodes(node) {
        Ok(_) => (),
        Err(_) => code = 1,
    }
    match reset_pools(pool) {
        Ok(_) => (),
        Err(_) => code = 1,
    }
    match reset_gupax_p2pool_api(gupax_p2pool_api) {
        Ok(_) => (),
        Err(_) => code = 1,
    }
    match code {
        0 => println!("\nGupaxx reset ... OK"),
        _ => eprintln!("\nGupaxx reset ... FAIL"),
    }
    exit(code);
}
