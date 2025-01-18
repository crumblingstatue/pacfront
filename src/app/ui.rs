use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    cmd::CmdBuf,
    eframe::egui,
    egui_colors::Colorix,
    egui_dock::{DockArea, DockState},
    std::process::Command,
    tabs::{Tab, TabViewState},
};

pub mod cmd;
mod paint_util;
mod tabs;

pub(super) struct UiState {
    dock_state: DockState<Tab>,
    pub shared: SharedUiState,
}

#[derive(Default)]
pub struct SharedUiState {
    cmd: CmdBuf,
    pub colorix: Option<Colorix>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(Tab::default_tabs()),
        }
    }
}

pub fn top_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .exact_height(26.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (re, painter) =
                    ui.allocate_painter(egui::vec2(24.0, 24.0), egui::Sense::hover());
                paint_util::draw_logo(&painter, re.rect.center(), 8.0);
                ui.label("Pacfront");
                ui.separator();
                ui.menu_button("☰ Preferences", |ui| {
                    if ui.button("🎨 Color theme").clicked() {
                        ui.close_menu();
                        app.ui.dock_state.push_to_first_leaf(Tab::ColorTheme);
                    }
                    match crate::config::cfg_dir() {
                        Some(dir) => {
                            if ui.button("Open config dir").clicked() {
                                ui.close_menu();
                                let _ = Command::new("xdg-open").arg(dir).status();
                            }
                        }
                        None => {
                            ui.label("<missing config dir>");
                        }
                    }
                });
            });
        });
}

pub fn central_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    DockArea::new(&mut app.ui.dock_state)
        .show_leaf_collapse_buttons(false)
        .show_leaf_close_all_buttons(false)
        .show(ctx, &mut TabViewState {
            pac: &mut app.pac,
            ui: &mut app.ui.shared,
        });
}
