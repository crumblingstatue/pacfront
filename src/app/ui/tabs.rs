use {
    super::{PacState, SharedUiState},
    eframe::egui,
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

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalPkgList(_) => {
                format!("Local packages ({})", self.pac.borrow_pkg_list().len()).into()
            }
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
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalPkgList(state) => local_pkg_list::ui(ui, self.pac, self.ui, state),
            Tab::RemotePkgList(state) => remote_pkg_list::ui(ui, self.pac, self.ui, state),
            Tab::Pkg(tab) => package::ui(ui, self.pac, self.ui, tab),
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
        }
    }
}

pub enum Tab {
    LocalPkgList(PkgListState),
    RemotePkgList(PkgListState),
    Pkg(PkgTab),
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
