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

use egui::Vec2;

use crate::{
    app::Benchmark,
    disk::{gupax_p2pool_api::GupaxP2poolApi, state::Status, status::*},
    helper::{
        node::PubNodeApi,
        p2pool::{ImgP2pool, PubP2poolApi},
        xrig::{
            xmrig::{ImgXmrig, PubXmrigApi},
            xmrig_proxy::PubXmrigProxyApi,
        },
        xvb::PubXvbApi,
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
        node_api: &Arc<Mutex<PubNodeApi>>,
        p2pool_api: &Arc<Mutex<PubP2poolApi>>,
        xmrig_api: &Arc<Mutex<PubXmrigApi>>,
        xmrig_proxy_api: &Arc<Mutex<PubXmrigProxyApi>>,
        xvb_api: &Arc<Mutex<PubXvbApi>>,
        p2pool_img: &Arc<Mutex<ImgP2pool>>,
        xmrig_img: &Arc<Mutex<ImgXmrig>>,
        node_alive: bool,
        p2pool_alive: bool,
        xmrig_alive: bool,
        xmrig_proxy_alive: bool,
        xvb_alive: bool,
        max_threads: usize,
        gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>,
        benchmarks: &[Benchmark],
        size: Vec2,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        //---------------------------------------------------------------------------------------------------- [Processes]
        if self.submenu == Submenu::Processes {
            self.processes(
                sys,
                size,
                ui,
                node_alive,
                node_api,
                p2pool_alive,
                p2pool_api,
                p2pool_img,
                xmrig_alive,
                xmrig_api,
                xmrig_proxy_alive,
                xmrig_proxy_api,
                xmrig_img,
                xvb_alive,
                xvb_api,
                max_threads,
            );
        //---------------------------------------------------------------------------------------------------- [P2Pool]
        } else if self.submenu == Submenu::P2pool {
            self.p2pool(size, ui, gupax_p2pool_api, p2pool_alive, p2pool_api);
        //---------------------------------------------------------------------------------------------------- [Benchmarks]
        } else if self.submenu == Submenu::Benchmarks {
            self.benchmarks(size, ui, benchmarks, xmrig_alive, xmrig_api)
        }
    }
}
