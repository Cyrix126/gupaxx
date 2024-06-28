use egui::TextStyle::{self, Name};
use egui::{RichText, SelectableLabel, TopBottomPanel};
use log::debug;

use crate::{app::Tab, utils::constants::SPACE};

impl crate::app::App {
    pub fn top_panel(&mut self, ctx: &egui::Context) {
        debug!("App | Rendering TOP tabs");
        TopBottomPanel::top("top").show(ctx, |ui| {
            let width = (self.size.x - (SPACE * 16.0)) / 7.0;
            let height = self.size.y / 15.0;
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
                        SelectableLabel::new(self.tab == Tab::Gupax, "Gupaxx"),
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
                let font_size = ui.text_style_height(&TextStyle::Name("Tab".into())) / 2.5;
                if ui
                    .add_sized(
                        [width, height],
                        SelectableLabel::new(
                            self.tab == Tab::XmrigProxy,
                            RichText::new("XMRig-Proxy").size(font_size),
                        ),
                    )
                    .clicked()
                {
                    self.tab = Tab::XmrigProxy;
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
