use crate::inits::init_text_styles;
use crate::SPACE;
use egui::Color32;
use log::debug;
use log::info;

use super::App;

impl App {
    pub fn resize(&mut self, ctx: &egui::Context) {
        // This resizes fonts/buttons/etc globally depending on the width.
        // This is separate from the [self.width != available_width] logic above
        // because placing [init_text_styles()] above would mean calling it 60x a second
        // while the user was readjusting the frame. It's a pretty heavy operation and looks
        // buggy when calling it that many times. Looking for a [must_resize] in addition to
        // checking if the user is hovering over the app means that we only have call it once.
        debug!("App | Checking if we need to resize");
        if self.must_resize && ctx.is_pointer_over_area() {
            self.resizing = true;
            self.must_resize = false;
        }
        // This (ab)uses [Area] and [TextEdit] to overlay a full black layer over whatever UI we had before.
        // It incrementally becomes more opaque until [self.alpha] >= 250, when we just switch to pure black (no alpha).
        // When black, we're safe to [init_text_styles()], and then incrementally go transparent, until we remove the layer.
        if self.resizing {
            egui::Area::new("resize_layer")
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(ctx, |ui| {
                    if self.alpha < 250 {
                        egui::Frame::none()
                            .fill(Color32::from_rgba_premultiplied(0, 0, 0, self.alpha))
                            .show(ui, |ui| {
                                ui.add_sized(
                                    [ui.available_width() + SPACE, ui.available_height() + SPACE],
                                    egui::TextEdit::multiline(&mut ""),
                                );
                            });
                        ctx.request_repaint();
                        self.alpha += 10;
                    } else {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(0, 0, 0))
                            .show(ui, |ui| {
                                ui.add_sized(
                                    [ui.available_width() + SPACE, ui.available_height() + SPACE],
                                    egui::TextEdit::multiline(&mut ""),
                                );
                            });
                        ctx.request_repaint();
                        info!(
                            "App | Resizing frame to match new internal resolution: [{}x{}]",
                            self.width, self.height
                        );
                        init_text_styles(ctx, self.width, self.state.gupax.selected_scale);
                        self.resizing = false;
                    }
                });
        } else if self.alpha != 0 {
            egui::Area::new("resize_layer")
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgba_premultiplied(0, 0, 0, self.alpha))
                        .show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width() + SPACE, ui.available_height() + SPACE],
                                egui::TextEdit::multiline(&mut ""),
                            );
                        })
                });
            self.alpha -= 10;
            ctx.request_repaint();
        }
    }
}
