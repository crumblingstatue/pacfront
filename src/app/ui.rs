use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    alpm::Package,
    eframe::egui,
    egui_dock::{DockArea, DockState, TabViewer},
    egui_extras::{Column, TableBuilder},
};

pub(super) struct UiState {
    dock_state: DockState<Tab>,
    shared: SharedUiState,
}

#[derive(Default)]
struct SharedUiState {
    filter_string: String,
    cmd: CmdBuf,
}

#[derive(Default)]
struct CmdBuf {
    cmds: Vec<Cmd>,
}

impl CmdBuf {
    fn push(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }
}

enum Cmd {
    OpenPkgTab(String),
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(vec![Tab::LocalDb]),
        }
    }
}

struct TabViewState<'pac, 'ui> {
    pac: &'pac PacState,
    ui: &'ui mut SharedUiState,
}

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalDb => "Local packages".into(),
            Tab::Package(name) => format!("Package '{name}'").into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalDb => package_list_ui(ui, self.pac, self.ui),
            Tab::Package(name) => package_ui(ui, self.pac, self.ui, name),
        }
    }
}

fn package_list_ui(ui: &mut egui::Ui, pac: &PacState, ui_state: &mut SharedUiState) {
    egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut ui_state.filter_string).hint_text("🔍 Filter"))
    });
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .auto_shrink(false)
        .striped(true)
        .header(32.0, |mut row| {
            row.col(|ui| {
                ui.label("Name");
            });
            row.col(|ui| {
                ui.label("Description");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            pac.with_pkg_list(|list| {
                let filtered: Vec<&Package> = list
                    .iter()
                    .filter(|pkg| {
                        pkg.name().contains(&ui_state.filter_string)
                            || pkg
                                .desc()
                                .is_some_and(|desc| desc.contains(&ui_state.filter_string))
                    })
                    .copied()
                    .collect();
                body.rows(24.0, filtered.len(), |mut row| {
                    let pkg = &filtered[row.index()];
                    row.col(|ui| {
                        if ui.link(pkg.name()).clicked() {
                            ui_state.cmd.push(Cmd::OpenPkgTab(pkg.name().to_string()));
                        }
                    });
                    row.col(|ui| {
                        ui.label(pkg.desc().unwrap_or("<missing description>"));
                    });
                });
            });
        });
}

fn package_ui(ui: &mut egui::Ui, pac: &PacState, ui_state: &mut SharedUiState, pkg_name: &str) {
    pac.with_pkg_list(
        |pkg_list| match pkg_list.iter().find(|pkg| pkg.name() == pkg_name) {
            Some(pkg) => {
                ui.heading(pkg.name());
                ui.label(pkg.desc().unwrap_or("<no description>"));
                ui.heading("Dependencies");
                for dep in pkg.depends() {
                    if ui.link(dep.name()).clicked() {
                        ui_state.cmd.push(Cmd::OpenPkgTab(dep.name().to_string()));
                    }
                }
            }
            None => {
                ui.label("<Unresolved package>");
            }
        },
    );
}

#[derive(Default)]
pub enum Tab {
    #[default]
    LocalDb,
    Package(String),
}

pub fn top_panel_ui(_app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.label("📦 Pacfront");
    });
}

pub fn central_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    DockArea::new(&mut app.ui.dock_state).show(ctx, &mut TabViewState {
        pac: &app.pac,
        ui: &mut app.ui.shared,
    });
}

pub fn process_cmds(app: &mut PacfrontApp, _ctx: &egui::Context) {
    for cmd in std::mem::take(&mut app.ui.shared.cmd.cmds) {
        match cmd {
            Cmd::OpenPkgTab(name) => app.ui.dock_state.push_to_first_leaf(Tab::Package(name)),
        }
    }
}
