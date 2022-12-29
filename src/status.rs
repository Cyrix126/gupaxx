// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janaiyo
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
	PubP2poolApi,
	PubXmrigApi,
	ImgP2pool,
	ImgXmrig,
	constants::*,
	Sys,
	Hash,
	Submenu,
	macros::*,
	GupaxP2poolApi,
	PayoutView,
};
use std::sync::{Arc,Mutex};
use log::*;
use egui::{
	Label,RichText,TextStyle,
	TextStyle::Monospace,
	TextStyle::Name,
	TextEdit,
	SelectableLabel,
};

impl crate::disk::Status {
pub fn show(&mut self, sys: &Arc<Mutex<Sys>>, p2pool_api: &Arc<Mutex<PubP2poolApi>>, xmrig_api: &Arc<Mutex<PubXmrigApi>>, p2pool_img: &Arc<Mutex<ImgP2pool>>, xmrig_img: &Arc<Mutex<ImgXmrig>>, p2pool_alive: bool, xmrig_alive: bool, max_threads: usize, gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>, width: f32, height: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
	//---------------------------------------------------------------------------------------------------- [Processes]
	if self.submenu == Submenu::Processes {
	let width = (width/3.0)-(SPACE*1.666);
	let min_height = height/1.1;
	let height = height/25.0;
	ui.horizontal(|ui| {
	// [Gupax]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [Gupax]");
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[Gupax]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("Gupax is online");
		ui.style_mut().override_text_style = Some(Monospace);
		let sys = lock!(sys);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_GUPAX_UPTIME);
		ui.add_sized([width, height], Label::new(sys.gupax_uptime.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_CPU_USAGE);
		ui.add_sized([width, height], Label::new(sys.gupax_cpu_usage.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
		ui.add_sized([width, height], Label::new(sys.gupax_memory_used_mb.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
		ui.add_sized([width, height], Label::new(sys.system_cpu_usage.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
		ui.add_sized([width, height], Label::new(sys.system_memory.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU Model").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
		ui.add_sized([width, height], Label::new(sys.system_cpu_model.to_string()));
		drop(sys);
	})});
	// [P2Pool]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [P2Pool]");
		ui.set_enabled(p2pool_alive);
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[P2Pool]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("P2Pool is online").on_disabled_hover_text("P2Pool is offline");
		let height = height/1.4;
		ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
		let api = lock!(p2pool_api);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_P2POOL_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("Shares Found").underline().color(BONE))).on_hover_text(STATUS_P2POOL_SHARES);
		ui.add_sized([width, height], Label::new(format!("{}", api.shares_found)));
		ui.add_sized([width, height], Label::new(RichText::new("Payouts").underline().color(BONE))).on_hover_text(STATUS_P2POOL_PAYOUTS);
		ui.add_sized([width, height], Label::new(format!("Total: {}", api.payouts)));
		ui.add_sized([width, height], Label::new(format!("[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]", api.payouts_hour, api.payouts_day, api.payouts_month)));
		ui.add_sized([width, height], Label::new(RichText::new("XMR Mined").underline().color(BONE))).on_hover_text(STATUS_P2POOL_XMR);
		ui.add_sized([width, height], Label::new(format!("Total: {:.13} XMR", api.xmr)));
		ui.add_sized([width, height], Label::new(format!("[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]", api.xmr_hour, api.xmr_day, api.xmr_month)));
		ui.add_sized([width, height], Label::new(RichText::new("Hashrate (15m/1h/24h)").underline().color(BONE))).on_hover_text(STATUS_P2POOL_HASHRATE);
		ui.add_sized([width, height], Label::new(format!("[{} H/s]\n[{} H/s]\n[{} H/s]", api.hashrate_15m, api.hashrate_1h, api.hashrate_24h)));
		ui.add_sized([width, height], Label::new(RichText::new("Miners Connected").underline().color(BONE))).on_hover_text(STATUS_P2POOL_CONNECTIONS);
		ui.add_sized([width, height], Label::new(format!("{}", api.connections)));
		ui.add_sized([width, height], Label::new(RichText::new("Effort").underline().color(BONE))).on_hover_text(STATUS_P2POOL_EFFORT);
		ui.add_sized([width, height], Label::new(format!("[Average: {}] [Current: {}]", api.average_effort, api.current_effort)));
		let img = lock!(p2pool_img);
		ui.add_sized([width, height], Label::new(RichText::new("Monero Node").underline().color(BONE))).on_hover_text(STATUS_P2POOL_MONERO_NODE);
		ui.add_sized([width, height], Label::new(format!("[IP: {}]\n[RPC: {}] [ZMQ: {}]", &img.host, &img.rpc, &img.zmq)));
		ui.add_sized([width, height], Label::new(RichText::new("Sidechain").underline().color(BONE))).on_hover_text(STATUS_P2POOL_POOL);
		ui.add_sized([width, height], Label::new(&img.mini));
		ui.add_sized([width, height], Label::new(RichText::new("Address").underline().color(BONE))).on_hover_text(STATUS_P2POOL_ADDRESS);
		ui.add_sized([width, height], Label::new(&img.address));
		drop(img);
		drop(api);
	})});
	// [XMRig]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [XMRig]");
		ui.set_enabled(xmrig_alive);
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[XMRig]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("XMRig is online").on_disabled_hover_text("XMRig is offline");
		ui.style_mut().override_text_style = Some(Monospace);
		let api = lock!(xmrig_api);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_XMRIG_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("CPU Load Averages").underline().color(BONE))).on_hover_text(STATUS_XMRIG_CPU);
		ui.add_sized([width, height], Label::new(format!("{}", api.resources)));
		ui.add_sized([width, height], Label::new(RichText::new("Hashrate Averages").underline().color(BONE))).on_hover_text(STATUS_XMRIG_HASHRATE);
		ui.add_sized([width, height], Label::new(format!("{}", api.hashrate)));
		ui.add_sized([width, height], Label::new(RichText::new("Difficulty").underline().color(BONE))).on_hover_text(STATUS_XMRIG_DIFFICULTY);
		ui.add_sized([width, height], Label::new(format!("{}", api.diff)));
		ui.add_sized([width, height], Label::new(RichText::new("Shares").underline().color(BONE))).on_hover_text(STATUS_XMRIG_SHARES);
		ui.add_sized([width, height], Label::new(format!("[Accepted: {}] [Rejected: {}]", api.accepted, api.rejected)));
		ui.add_sized([width, height], Label::new(RichText::new("Pool").underline().color(BONE))).on_hover_text(STATUS_XMRIG_POOL);
		ui.add_sized([width, height], Label::new(&lock!(xmrig_img).url));
		ui.add_sized([width, height], Label::new(RichText::new("Threads").underline().color(BONE))).on_hover_text(STATUS_XMRIG_THREADS);
		ui.add_sized([width, height], Label::new(format!("{}/{}", &lock!(xmrig_img).threads, max_threads)));
		drop(api);
	})});
	});
	//---------------------------------------------------------------------------------------------------- [P2Pool]
	} else if self.submenu == Submenu::P2pool {
	let mut api = lock!(gupax_p2pool_api);
	let text = height / 25.0;
	let log = height / 2.4;
	ui.style_mut().override_text_style = Some(Monospace);
	// Payout Text + PayoutView buttons
	ui.group(|ui| {
		ui.horizontal(|ui| {
			let width = (width/3.0)-(SPACE*4.0);
			ui.add_sized([width, text], Label::new(RichText::new(format!("Total Payouts: {}", api.payout)).underline().color(LIGHT_GRAY))).on_hover_text(STATUS_SUBMENU_PAYOUT);
			ui.separator();
			ui.add_sized([width, text], Label::new(RichText::new(format!("Total XMR: {}", api.xmr)).underline().color(LIGHT_GRAY))).on_hover_text(STATUS_SUBMENU_XMR);
			let width = width / 4.0;
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Latest, "Latest")).on_hover_text(STATUS_SUBMENU_LATEST).clicked() { self.payout_view = PayoutView::Latest; }
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Oldest, "Oldest")).on_hover_text(STATUS_SUBMENU_OLDEST).clicked() { self.payout_view = PayoutView::Oldest; }
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Biggest, "Biggest")).on_hover_text(STATUS_SUBMENU_BIGGEST).clicked() {
				api.update_payout_high();
				self.payout_view = PayoutView::Biggest;
			}
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Smallest, "Smallest")).on_hover_text(STATUS_SUBMENU_SMALLEST).clicked() {
				api.update_payout_low();
				self.payout_view = PayoutView::Smallest;
			}
		});
		ui.separator();
		// Actual logs
		egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
			egui::ScrollArea::vertical().stick_to_bottom(self.payout_view == PayoutView::Oldest).max_width(width).max_height(log).auto_shrink([false; 2]).show_viewport(ui, |ui, _| {
				match self.payout_view {
					PayoutView::Latest   => ui.add_sized([width, log], TextEdit::multiline(&mut api.log.as_str())),
					PayoutView::Oldest   => ui.add_sized([width, log], TextEdit::multiline(&mut api.log_rev.as_str())),
					PayoutView::Biggest  => ui.add_sized([width, log], TextEdit::multiline(&mut api.payout_high.as_str())),
					PayoutView::Smallest => ui.add_sized([width, log], TextEdit::multiline(&mut api.payout_low.as_str())),
				};
			});
		});
	});
	drop(api);
	//---------------------------------------------------------------------------------------------------- [Monero]
	} else if self.submenu == Submenu::Monero {
	}
}
}
