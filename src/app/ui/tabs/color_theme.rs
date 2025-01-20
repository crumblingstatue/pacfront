use {
    eframe::egui,
    egui_colors::{Colorix, tokens::ThemeColor},
};

pub fn ui(ui: &mut egui::Ui, opt_colorix: &mut Option<Colorix>) {
    match opt_colorix {
        Some(colorix) => {
            if ui.button("Deactivate custom colors").clicked() {
                ui.ctx().style_mut(|style| {
                    *style = egui::Style::default();
                });
                *opt_colorix = None;
                return;
            }
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.label("Light/dark");
                    colorix.light_dark_toggle_button(ui);
                });
                ui.group(|ui| {
                    ui.label("Preset");
                    colorix.themes_dropdown(
                        ui,
                        Some((vec!["PacArch"], vec![THEME_PACARCH])),
                        false,
                    );
                });
            });
            ui.add_space(8.0);
            colorix.ui_combo_12(ui, true);
        }
        None => {
            if ui.button("Activate custom colors").clicked() {
                *opt_colorix = Some(Colorix::init(ui.ctx(), THEME_PACARCH));
            }
        }
    }
}

const THEME_PACARCH: egui_colors::Theme = [
    ThemeColor::Custom([0, 125, 255]),
    ThemeColor::Custom([119, 164, 255]),
    ThemeColor::Indigo,
    ThemeColor::Iris,
    ThemeColor::Indigo,
    ThemeColor::Gray,
    ThemeColor::Iris,
    ThemeColor::Indigo,
    ThemeColor::Blue,
    ThemeColor::Indigo,
    ThemeColor::Custom([254, 247, 116]),
    ThemeColor::Custom([0, 245, 232]),
];
