use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    cmd::CmdBuf,
    eframe::egui,
    egui_dock::{DockArea, DockState, TabViewer},
    tabs::package::PkgTab,
};

mod tabs {
    pub mod local_pkg_list;
    pub mod package;
    pub mod remote_pkg_list;
}
pub mod cmd;

pub(super) struct UiState {
    dock_state: DockState<Tab>,
    shared: SharedUiState,
}

#[derive(Default)]
struct SharedUiState {
    filter_string: String,
    cmd: CmdBuf,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(vec![Tab::LocalDb, Tab::SyncDbList, Tab::SyncDbPkgList]),
        }
    }
}

struct TabViewState<'pac, 'ui> {
    pac: &'pac mut PacState,
    ui: &'ui mut SharedUiState,
}

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalDb => format!("Local packages ({})", self.pac.borrow_pkg_list().len()).into(),
            Tab::SyncDbPkgList => format!(
                "Remote packages ({})",
                self.pac
                    .borrow_sync()
                    .iter()
                    .map(|db| db.pkgs().len())
                    .sum::<usize>()
            )
            .into(),
            Tab::LocalPkg(pkg) => format!("Package '{}'", pkg.name).into(),
            Tab::RemotePkg(pkg) => format!("Remote Package '{}'", pkg.name).into(),
            Tab::SyncDbList => format!("Sync DBs ({})", self.pac.borrow_sync().len()).into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalDb => tabs::local_pkg_list::ui(ui, self.pac, self.ui),
            Tab::SyncDbPkgList => tabs::remote_pkg_list::ui(ui, self.pac, self.ui),
            Tab::LocalPkg(tab) => tabs::package::ui(ui, self.pac, self.ui, tab, false),
            Tab::RemotePkg(tab) => tabs::package::ui(ui, self.pac, self.ui, tab, true),
            Tab::SyncDbList => syncdb_list_ui(ui, self.pac, self.ui),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        #[expect(clippy::match_like_matches_macro)]
        match tab {
            Tab::LocalDb => false,
            _ => true,
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            Tab::LocalDb => false,
            Tab::LocalPkg(pkg_tab) => pkg_tab.force_close,
            Tab::RemotePkg(pkg_tab) => pkg_tab.force_close,
            Tab::SyncDbList => false,
            Tab::SyncDbPkgList => false,
        }
    }
}

#[derive(Default)]
pub enum Tab {
    #[default]
    LocalDb,
    SyncDbList,
    SyncDbPkgList,
    LocalPkg(PkgTab),
    RemotePkg(PkgTab),
}

pub fn top_panel_ui(_app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.label("📦 Pacfront");
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

fn syncdb_list_ui(ui: &mut egui::Ui, pac: &mut PacState, _ui_state: &mut SharedUiState) {
    pac.with_sync_mut(|sync| {
        for db in sync {
            ui.label(format!("{} ({})", db.name(), db.pkgs().len()));
        }
    });
}
