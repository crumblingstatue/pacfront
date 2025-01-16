use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    alpm::Package,
    eframe::egui,
    egui_dock::{DockArea, DockState, TabViewer},
    egui_extras::{Column, TableBuilder},
    std::process::Command,
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
            Tab::Package(pkg) => format!("Package '{}'", pkg.name).into(),
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

fn package_ui(
    ui: &mut egui::Ui,
    pac: &PacState,
    ui_state: &mut SharedUiState,
    pkg_tab: &mut PkgTab,
) {
    pac.with_pkg_list(
        |pkg_list| match pkg_list.iter().find(|pkg| pkg.name() == pkg_tab.name) {
            Some(pkg) => {
                ui.heading(pkg.name());
                ui.separator();
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::General, "General");
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::Files, "File list");
                });
                ui.separator();
                match pkg_tab.tab {
                    PkgTabTab::General => {
                        ui.label(pkg.desc().unwrap_or("<no description>"));
                        let deps = pkg.depends();
                        ui.heading("Dependencies");
                        if deps.is_empty() {
                            ui.label("<this package has no dependencies>");
                        } else {
                            for dep in deps {
                                if ui.link(dep.name()).clicked() {
                                    ui_state.cmd.push(Cmd::OpenPkgTab(dep.name().to_string()));
                                }
                            }
                        }
                    }
                    PkgTabTab::Files => {
                        for file in pkg.files().files() {
                            let name = format!("/{}", file.name());
                            if ui.link(&name).clicked() {
                                Command::new("xdg-open").arg(name).status().unwrap();
                            }
                        }
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
    Package(PkgTab),
}

pub struct PkgTab {
    name: String,
    tab: PkgTabTab,
}

#[derive(PartialEq)]
enum PkgTabTab {
    General,
    Files,
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
            Cmd::OpenPkgTab(name) => app.ui.dock_state.push_to_first_leaf(Tab::Package(PkgTab {
                name,
                tab: PkgTabTab::General,
            })),
        }
    }
}
