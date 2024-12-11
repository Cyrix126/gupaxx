use egui::{Hyperlink, Image, Separator, TextStyle, TextWrapMode, Ui};
use log::debug;

use crate::SPACE;
/// logo first, first hyperlink will be the header, description under.
/// will take care of centering widgets if boerder weight is more than 0.
#[allow(clippy::needless_range_loop)]
pub fn header_tab(
    ui: &mut Ui,
    logo: Option<Image>,
    // text, link, hover text
    links: &[(&str, &str, &str)],
    subtitle: Option<String>,
    one_line_center: bool,
) {
    // width - logo and links and separators divided by double the size of logo (can't know width of links).
    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    ui.style_mut().override_text_style = Some(TextStyle::Heading);
    ui.add_space(SPACE);
    if one_line_center {
        let height = 64.0;
        let nb_links = links.len();
        let border_weight = ((ui.available_width()
            - ((height * 4.0 * nb_links as f32) + if logo.is_some() { height * 2.0 } else { 0.0 }))
            / (height * 2.0))
            .max(0.0) as usize;
        // nb_columns add logo if exist plus number of links with separator for each + number of column for border space
        let nb_columns = if logo.is_some() { 1 } else { 0 } + (links.len() * 2) + border_weight * 2;
        ui.columns(nb_columns, |col| {
            // first column for left border
            for n in 0..(border_weight) {
                col[n].vertical_centered(|ui| ui.add_space(0.0));
                debug!("left side space: {}", n);
            }
            // jump first column, stop less 2, because begin at 0 and end with space column.
            let mut nb_col = border_weight;
            if let Some(logo) = logo {
                debug!("logo: {}", nb_col);
                col[nb_col].vertical_centered(|ui| ui.add_sized([height, height], logo));
                nb_col += 1;
            }
            for link in links {
                debug!("separator: {}", nb_col);
                col[nb_col].vertical_centered(|ui| {
                    ui.add_sized(
                        [height / 8.0, height],
                        Separator::default().vertical().spacing(height / 8.0),
                    )
                });
                nb_col += 1;

                debug!("link: {}", nb_col);
                col[nb_col].vertical_centered(|ui| {
                    ui.add_sized(
                        [ui.available_width(), height],
                        Hyperlink::from_label_and_url(link.0, link.1),
                    );
                });
                nb_col += 1;
            }

            for n in nb_col..(nb_col + border_weight) {
                debug!("right side border space: {}", n);
                col[n].vertical_centered(|ui| ui.add_space(0.0));
            }
        });
    } else {
        // top down
        ui.vertical_centered(|ui| {
            if let Some(source) = logo {
                ui.add(source);
            }
            for link in links {
                ui.hyperlink_to(link.0, link.1);
            }
            ui.style_mut().override_text_style = Some(TextStyle::Body);
        });
    }
    if let Some(desc) = subtitle {
        ui.label(desc);
    }
    ui.add_space(SPACE);
}
