use egui::{Hyperlink, Image, Label, Separator, TextStyle, TextWrapMode, Ui};

use crate::SPACE;
// prevent compiling if no elements are added to a header. No need for a header then.
const fn check_header_element(is_logo_none: bool, is_links_empty: bool, is_subtitle_none: bool) {
    if is_logo_none && is_links_empty && is_subtitle_none {
        panic!("header_tab must be used with at least one element");
    }
}
/// logo first, first hyperlink will be the header, description under.
/// will take care of centering widgets if boerder weight is more than 0.
#[allow(clippy::needless_range_loop)]
pub fn header_tab(
    ui: &mut Ui,
    logo: Option<Image>,
    // text, link, hover text
    links: &[(&str, &str, &str)],
    subtitle: Option<&str>,
    one_line_center: bool,
) {
    check_header_element(logo.is_none(), links.is_empty(), subtitle.is_none());
    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    ui.style_mut().override_text_style = Some(TextStyle::Heading);
    ui.add_space(SPACE);
    if one_line_center {
        ui.spacing_mut().item_spacing.x = 0.0;
        let height_logo = 64.0;
        let width_links = links
            .iter()
            .map(|x| x.0.len() as f32 * ui.text_style_height(&TextStyle::Heading) / 2.0)
            .collect::<Vec<f32>>();
        let width_subtitle = subtitle.unwrap_or_default().len() as f32
            * ui.text_style_height(&TextStyle::Body)
            / 2.0;
        let nb_txt = links.len() + if subtitle.is_some() { 1 } else { 0 };
        // width of separator depends of width of ui and number of texts
        let width_separator = (ui.available_width() / ui.text_style_height(&TextStyle::Heading)
            * 4.0)
            / nb_txt as f32;
        // width available - logo and separator - total width of txt - separator for each text then divided by two
        let border_width = (((ui.available_width()
            - if logo.is_some() {
                height_logo + width_separator
            } else {
                0.0
            }
            - width_links.iter().sum::<f32>()
            - width_subtitle
            - (nb_txt as f32 * width_separator))
            + width_separator)
            / 2.0)
            .max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(border_width);
            if let Some(logo) = logo {
                ui.add_sized([height_logo, height_logo], logo);
                ui.add_sized(
                    [width_separator, height_logo],
                    Separator::default().vertical().spacing(width_separator),
                );
            }
            for (count, link) in links.iter().enumerate() {
                ui.add_sized(
                    // [width_links[count], height_logo],
                    [0.0, height_logo],
                    Hyperlink::from_label_and_url(link.0, link.1),
                );
                if count != (links.len() - 1) || subtitle.is_some() {
                    ui.add_sized(
                        [0.0, height_logo],
                        Separator::default().vertical().spacing(width_separator),
                    );
                }
            }
            if let Some(desc) = subtitle {
                ui.style_mut().override_text_style = Some(TextStyle::Body);
                ui.add_sized([0.0, height_logo], Label::new(desc));
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
            if let Some(desc) = subtitle {
                ui.style_mut().override_text_style = Some(TextStyle::Body);
                ui.label(desc);
            }
        });
    }
    ui.add_space(SPACE);
}
