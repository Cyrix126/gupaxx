// Gupaxx - Fork of Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//---------------------------------------------------------------------------------------------------- Use
use crate::constants::{
    COMMIT, GUPAX_VERSION, NODE_VERSION, OS_NAME, P2POOL_VERSION, XMRIG_VERSION,
};

//----------------------------------------------------------------------------------------------------
#[cold]
#[inline(never)]
/// Set custom panic hook.
pub(crate) fn set_panic_hook(now: std::time::Instant) {
    std::panic::set_hook(Box::new(move |panic_info| {
        // Set stack-trace.
        let stack_trace = std::backtrace::Backtrace::force_capture();
        let args = std::env::args_os();
        let uptime = now.elapsed().as_secs_f32();
        let panic_msg = panic_info.to_string();
        // Re-format panic info.
        let panic_info = format!(
            "panic error: {panic_msg}\n
            {panic_info:#?}

info:
   OS          | {OS_NAME}
   args        | {args:?}
   commit      | {COMMIT}
   gupaxx      | {GUPAX_VERSION}
   monerod     | {NODE_VERSION} (bundled) 
   p2pool      | {P2POOL_VERSION} (bundled)
   xmrig       | {XMRIG_VERSION} (bundled)
   xmrig-proxy | {XMRIG_VERSION} (bundled)
   uptime      | {uptime} seconds

stack backtrace:\n{stack_trace}",
        );

        // Attempt to write panic info to disk.
        match crate::disk::get_gupax_data_path() {
            Ok(mut path) => {
                path.push("crash.txt");
                match std::fs::write(&path, &panic_info) {
                    Ok(_) => {
                        eprintln!("\nmass_panic!() - Saved panic log to: {}\n", path.display())
                    }
                    Err(e) => eprintln!("\nmass_panic!() - Could not save panic log: {e}\n"),
                }
            }
            Err(e) => eprintln!("panic_hook PATH error: {e}"),
        }

        // Exit all threads.
        benri::mass_panic!(panic_info);
    }));
}
