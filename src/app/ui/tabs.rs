use {
    super::{PacState, SharedUiState},
    eframe::egui,
    egui_colors::{Colorix, tokens::ThemeColor},
    egui_dock::TabViewer,
    package::PkgTab,
};

pub mod local_pkg_list;
pub mod package;
pub mod remote_pkg_list;

pub struct TabViewState<'pac, 'ui> {
    pub pac: &'pac mut PacState,
    pub ui: &'ui mut SharedUiState,
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

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalPkgList(_) => format!(
                "Local packages ({})",
                self.pac.borrow_local_pkg_list().len()
            )
            .into(),
            Tab::RemotePkgList(_) => format!(
                "Remote packages ({})",
                self.pac
                    .borrow_sync()
                    .iter()
                    .map(|db| db.pkgs().len())
                    .sum::<usize>()
            )
            .into(),
            Tab::Pkg(pkg) => format!("📦 {}", pkg.id).into(),
            Tab::ColorTheme => "🎨 Color theme".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalPkgList(state) => local_pkg_list::ui(ui, self.pac, self.ui, state),
            Tab::RemotePkgList(state) => remote_pkg_list::ui(ui, self.pac, self.ui, state),
            Tab::Pkg(tab) => package::ui(ui, self.pac, self.ui, tab),
            Tab::ColorTheme => match &mut self.ui.colorix {
                Some(colorix) => {
                    if ui.button("Deactivate custom colors").clicked() {
                        ui.ctx().style_mut(|style| {
                            *style = egui::Style::default();
                        });
                        self.ui.colorix = None;
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
                        self.ui.colorix = Some(Colorix::init(ui.ctx(), THEME_PACARCH));
                    }
                }
            },
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        #[expect(clippy::match_like_matches_macro)]
        match tab {
            Tab::LocalPkgList(_) | Tab::RemotePkgList(_) => false,
            _ => true,
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            Tab::LocalPkgList(_) => false,
            Tab::Pkg(pkg_tab) => pkg_tab.force_close,
            Tab::RemotePkgList(_) => false,
            Tab::ColorTheme => false,
        }
    }
}

pub enum Tab {
    LocalPkgList(PkgListState),
    RemotePkgList(PkgListState),
    Pkg(PkgTab),
    ColorTheme,
}
impl Tab {
    pub(crate) fn default_tabs() -> Vec<Tab> {
        vec![
            Tab::LocalPkgList(PkgListState::default()),
            Tab::RemotePkgList(PkgListState::default()),
        ]
    }
}

#[derive(Default)]
pub struct PkgListState {
    filter_string: String,
}
