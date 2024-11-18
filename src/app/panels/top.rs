use crate::app::Tab;
use egui::TextStyle;
use egui::{ScrollArea, SelectableLabel, Separator, TopBottomPanel, Ui};
use log::debug;
use strum::{EnumCount, IntoEnumIterator};

impl crate::app::App {
    pub fn top_panel(&mut self, ctx: &egui::Context) {
        debug!("App | Rendering TOP tabs");
        TopBottomPanel::top("top").show(ctx, |ui| {
            // low spacing to shrink and be able to show all tabs on one line on 640x480
            ui.style_mut().spacing.item_spacing.x = 4.0;
            // spacing of separator, will reduce width size of the button. Low value so that tabs can be selected easily.
            let spacing_separator = 2.0;
            // TODO if screen smaller, go on two lines.
            // TODO if screen really to small, go on tab per line.
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                ui.style_mut().override_text_style = Some(TextStyle::Heading);
                let height = ui
                    .style()
                    .text_styles
                    .get(&TextStyle::Heading)
                    .unwrap()
                    .size
                    * 2.75;
                // width = (width - / number of tab) - (space between widget * 2.0 + space of separator / 2.0)
                let width = (((self.size.x) / Tab::COUNT as f32)
                    - ((ui.style().spacing.item_spacing.x * 2.0) + (spacing_separator / 2.0)))
                    .max(0.0);
                // height of tab menu relative to size of text. coeff 2.75 is arbitrary but good enough to be easily clickable.
                self.tabs(ui, [width, height], spacing_separator);
            });
        });
    }

    fn tabs(&mut self, ui: &mut Ui, size: [f32; 2], spacing_separator: f32) {
        ScrollArea::horizontal()
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                for (count, tab) in Tab::iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            // we don't want y item spacing to influence the added space
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            ui.add_space(spacing_separator);
                            ui.horizontal(|ui| {
                                if ui
                                    .add_sized(
                                        size,
                                        SelectableLabel::new(self.tab == tab, tab.to_string()),
                                    )
                                    .clicked()
                                {
                                    self.tab = tab
                                }
                            });
                            // add a space to prevent selectable button to be at the same line as the end of the top bar. Make it the same spacing as separators.
                            ui.add_space(spacing_separator);
                        });
                        if count + 1 != Tab::COUNT {
                            ui.add(Separator::default().spacing(spacing_separator).vertical());
                        }
                    });
                }
            });
    }
}
