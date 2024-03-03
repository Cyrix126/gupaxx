use egui::{Hyperlink, Image};

use crate::constants::{BYTES_XVB, SPACE};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    pub fn show(width: f32, height: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
        let website_height = height / 10.0;
        let width = width - SPACE;
        // ui.add_sized(
        //     [width, website_height],
        //     Hyperlink::from_label_and_url("XMRvsBeast", "https://xmrvsbeast.com"),
        // );
        ui.vertical_centered(|ui| {
            ui.add_sized(
                [width, website_height],
                Image::from_bytes("bytes:/xvb.png", BYTES_XVB),
            );
            ui.add_sized(
                [width / 8.0, website_height],
                Hyperlink::from_label_and_url("XMRvsBeast", "https://xmrvsbeast.com"),
            );
        });
    }
}
