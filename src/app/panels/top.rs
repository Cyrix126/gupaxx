use egui::TextStyle::Name;
use egui::{SelectableLabel, TopBottomPanel};
use log::debug;

use crate::{app::Tab, utils::constants::SPACE};

impl crate::app::App {
    pub fn top_panel(&mut self, ctx: &egui::Context) {
        debug!("App | Rendering TOP tabs");
        TopBottomPanel::top("top").show(ctx, |ui| {
            let width = (self.width - (SPACE * 11.0)) / 6.0;
            let height = self.height / 15.0;
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.style_mut().override_text_style = Some(Name("Tab".into()));
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::About, "About"),
                    )
                    .clicked()
                {
                    self.tab = Tab::About;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::Status, "Status"),
                    )
                    .clicked()
                {
                    self.tab = Tab::Status;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::Gupax, "Gupax"),
                    )
                    .clicked()
                {
                    self.tab = Tab::Gupax;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::P2pool, "P2Pool"),
                    )
                    .clicked()
                {
                    self.tab = Tab::P2pool;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::Xmrig, "XMRig"),
                    )
                    .clicked()
                {
                    self.tab = Tab::Xmrig;
                }
                ui.separator();
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(self.tab == Tab::Xvb, "XvB"),
                    )
                    .clicked()
                {
                    self.tab = Tab::Xvb;
                }
            });
            ui.add_space(4.0);
        });
    }
}
