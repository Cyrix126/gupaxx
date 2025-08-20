use egui::{TextStyle, Ui};

pub fn height_dropdown(nb_item: usize, ui: &Ui) -> f32 {
    nb_item as f32
        * (ui.text_style_height(&TextStyle::Button)
            + (ui.spacing().button_padding.y * 2.0)
            + ui.spacing().item_spacing.y)
}
