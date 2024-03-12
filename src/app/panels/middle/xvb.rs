use std::sync::{Arc, Mutex};

use egui::TextStyle::Name;
use egui::{Hyperlink, Image, TextEdit, Vec2};
use log::debug;

use crate::helper::xvb::PubXvbApi;
use crate::utils::macros::lock;
use crate::{
    constants::{BYTES_XVB, SPACE},
    utils::constants::{DARK_GRAY, XVB_URL},
};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    pub fn show(size: Vec2, _ctx: &egui::Context, ui: &mut egui::Ui, api: &Arc<Mutex<PubXvbApi>>) {
        let website_height = size.y / 10.0;
        // let width = size.x - SPACE;
        // let height = size.y - SPACE;
        let height = size.y;
        let width = size.x;
        let text_edit = size.y / 25.0;
        // logo and website link
        ui.vertical_centered(|ui| {
            ui.add_sized(
                [width, website_height],
                Image::from_bytes("bytes:/xvb.png", BYTES_XVB),
            );
            ui.add_sized(
                [width / 8.0, website_height],
                Hyperlink::from_label_and_url("XMRvsBeast", XVB_URL),
            );
        });
        // console output for log
        debug!("XvB Tab | Rendering [Console]");
        ui.group(|ui| {
            let height = size.y / 2.8;
            let width = size.x - SPACE;
            egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_width(width)
                    .max_height(height)
                    .auto_shrink([false; 2])
                    .show_viewport(ui, |ui, _| {
                        ui.add_sized(
                            [width, height],
                            TextEdit::multiline(&mut lock!(api).output.as_str()),
                        );
                    });
            });
            // address check
            // input token
        });
    }
}
