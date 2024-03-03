// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022-2023 hinto-janai
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

use crate::{
    app::Benchmark,
    disk::{gupax_p2pool_api::GupaxP2poolApi, state::Status, status::*},
    helper::{
        p2pool::{ImgP2pool, PubP2poolApi},
        xmrig::{ImgXmrig, PubXmrigApi},
        Sys,
    },
};
use std::sync::{Arc, Mutex};

mod benchmarks;
mod p2pool;
mod processes;

impl Status {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        sys: &Arc<Mutex<Sys>>,
        p2pool_api: &Arc<Mutex<PubP2poolApi>>,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
        p2pool_img: &Arc<Mutex<ImgP2pool>>,
        xmrig_img: &Arc<Mutex<ImgXmrig>>,
        p2pool_alive: bool,
        xmrig_alive: bool,
        max_threads: usize,
        gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>,
        benchmarks: &[Benchmark],
        width: f32,
        height: f32,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        //---------------------------------------------------------------------------------------------------- [Processes]
        if self.submenu == Submenu::Processes {
            self.processes(
                sys,
                width,
                height,
                ui,
                p2pool_alive,
                p2pool_api,
                xmrig_alive,
                xmrig_api,
                p2pool_img,
                xmrig_img,
                max_threads,
            );
        //---------------------------------------------------------------------------------------------------- [P2Pool]
        } else if self.submenu == Submenu::P2pool {
            self.p2pool(
                width,
                height,
                ui,
                gupax_p2pool_api,
                p2pool_alive,
                p2pool_api,
            );
        //---------------------------------------------------------------------------------------------------- [Benchmarks]
        } else if self.submenu == Submenu::Benchmarks {
            self.benchmarks(width, height, ui, benchmarks, xmrig_alive, xmrig_api)
        }
    }
}
