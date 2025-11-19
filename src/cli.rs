use clap::Parser;
use clap::Subcommand;
use clap::crate_authors;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use log::debug;
use log::info;
use log::warn;
use std::process::exit;

use crate::app::App;
use crate::miscs::print_disk_file;
use crate::miscs::print_gupax_p2pool_api;
use crate::resets::reset;
use crate::resets::reset_gupax_p2pool_api;
use crate::resets::reset_nodes;
use crate::resets::reset_pools;
use crate::resets::reset_state;

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(author = crate_authors!())]
#[command(version = crate_version!())]
#[command(about = crate_description!(), long_about = None)]
#[command(next_line_help = true)]
#[group(required = false, multiple = false)]
pub struct Cli {
    #[command(subcommand)]
    pub info: Option<GupaxxData>,
    #[clap(long, short, action)]
    pub logfile: bool,
    #[clap(long, action)]
    pub daemon: bool,
}

#[derive(Subcommand)]
pub enum GupaxxData {
    #[command(about = "Print Gupaxx state")]
    State,
    #[command(about = "Print the manual node list")]
    Nodes,
    #[command(about = "Print the P2Pool payout log, payout count, and total XMR mined")]
    Payouts,
    #[command(about = "Reset all Gupaxxstate (your settings)")]
    ResetState,
    #[command(about = "Reset the manual node list in the [P2Pool] tab")]
    ResetNodes,
    #[command(about = "Reset the manual pool list in the [XMRig] tab")]
    ResetPools,
    #[command(about = "Reset the permanent P2Pool stats that appear in the [Status] tab")]
    ResetPayouts,
    #[command(about = "Reset all Gupaxx state (your settings)")]
    ResetAll,
    #[command(
        about = "Disable all auto-startup settings for this instance (auto-update, auto-ping, etc)",
        name = "no-startup"
    )]
    Nostartup,
}
// #[cold]
// #[inline(never)]
pub fn parse_args<S: Into<String>>(mut app: App, args: &Cli, panic: S) -> App {
    info!("Parsing CLI arguments...");

    // Abort on panic
    let panic = panic.into();
    if !panic.is_empty() {
        warn!("[Gupax error] {panic}");
        exit(1);
    }
    if let Some(arg) = &args.info {
        match arg {
            GupaxxData::State => {
                debug!("Printing state...");
                print_disk_file(&app.state_path);
                exit(0);
            }
            GupaxxData::Nodes => {
                debug!("Printing node list...");
                print_disk_file(&app.node_path);
                exit(0);
            }
            GupaxxData::Payouts => {
                debug!("Printing payouts...\n");
                print_gupax_p2pool_api(&app.gupax_p2pool_api);
                exit(0);
            }
            GupaxxData::ResetState => {
                if let Ok(()) = reset_state(&app.state_path) {
                    println!("\nState reset ... OK");
                    exit(0);
                } else {
                    eprintln!("\nState reset ... FAIL");
                    exit(1)
                }
            }
            GupaxxData::ResetNodes => {
                if let Ok(()) = reset_nodes(&app.node_path) {
                    println!("\nNode reset ... OK");
                    exit(0)
                } else {
                    eprintln!("\nNode reset ... FAIL");
                    exit(1)
                }
            }
            GupaxxData::ResetPools => {
                if let Ok(()) = reset_pools(&app.pool_path) {
                    println!("\nPool reset ... OK");
                    exit(0)
                } else {
                    eprintln!("\nPool reset ... FAIL");
                    exit(1)
                }
            }
            GupaxxData::ResetPayouts => {
                if let Ok(()) = reset_gupax_p2pool_api(&app.gupax_p2pool_api_path) {
                    println!("\nGupaxP2poolApi reset ... OK");
                    exit(0)
                } else {
                    eprintln!("\nGupaxP2poolApi reset ... FAIL");
                    exit(1)
                }
            }
            GupaxxData::ResetAll => reset(
                &app.os_data_path,
                &app.state_path,
                &app.node_path,
                &app.pool_path,
                &app.gupax_p2pool_api_path,
            ),
            GupaxxData::Nostartup => app.no_startup = true,
        }
    }
    app
}
