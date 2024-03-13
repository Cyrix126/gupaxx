use std::sync::{Arc, Mutex};

use egui::TextStyle::Name;
use egui::{Hyperlink, Image, Label, RichText, TextEdit, Vec2};
use log::debug;

use crate::helper::xvb::PubXvbApi;
use crate::utils::constants::{GREEN, LIGHT_GRAY, ORANGE, RED, XVB_HELP, XVB_TOKEN_LEN};
use crate::utils::macros::lock;
use crate::utils::regex::Regexes;
use crate::{
    constants::{BYTES_XVB, SPACE},
    utils::constants::{DARK_GRAY, XVB_URL},
};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    pub fn show(
        &mut self,
        size: Vec2,
        address: &str,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        api: &Arc<Mutex<PubXvbApi>>,
    ) {
        let website_height = size.y / 10.0;
        // let width = size.x - SPACE;
        // let height = size.y - SPACE;
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
        });
        // input token
        let len_token = format!("{}", self.token.len());
        let text_check;
        let color;
        if self.token.is_empty() {
            text_check = format!("[{}/{}] ➖", len_token, XVB_TOKEN_LEN);
            color = LIGHT_GRAY;
        } else if self.token.parse::<u32>().is_ok() && self.token.len() < XVB_TOKEN_LEN {
            text_check = format!("[{}/{}] ", len_token, XVB_TOKEN_LEN);
            color = GREEN;
        } else if self.token.parse::<u32>().is_ok() && self.token.len() == XVB_TOKEN_LEN {
            text_check = "✔".to_string();
            color = GREEN;
        } else {
            text_check = format!("[{}/{}] ❌", len_token, XVB_TOKEN_LEN);
            color = RED;
        }
        ui.group(|ui| {
            let width = width - SPACE;
            ui.spacing_mut().text_edit_width = (width) - (SPACE * 3.0);
            ui.label("Your Token:");
            ui.horizontal(|ui| {
                ui.add_sized(
                    [width / 8.0, text_edit],
                    TextEdit::singleline(&mut self.token),
                );

                ui.add(Label::new(RichText::new(text_check).color(color)))
            });
        })
        .response
        .on_hover_text_at_pointer(XVB_HELP);
        // need to warn the user if no address is set in p2pool tab
        if !Regexes::addr_ok(address) {
            debug!("XvB Tab | Rendering warning text");
            ui.label(RichText::new("You don't have any payout address set in the P2pool Tab !\nXvB process needs one to function properly.")
                        .color(ORANGE));
        }
        // hero option
        // private stats
    }
}
