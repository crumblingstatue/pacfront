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
            Tab::LocalPkgList => {
                format!("Local packages ({})", self.pac.borrow_pkg_list().len()).into()
            }
            Tab::RemotePkgList => format!(
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
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalPkgList => local_pkg_list::ui(ui, self.pac, self.ui),
            Tab::RemotePkgList => remote_pkg_list::ui(ui, self.pac, self.ui),
            Tab::LocalPkg(tab) => package::ui(ui, self.pac, self.ui, tab, false),
            Tab::RemotePkg(tab) => package::ui(ui, self.pac, self.ui, tab, true),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        #[expect(clippy::match_like_matches_macro)]
        match tab {
            Tab::LocalPkgList | Tab::RemotePkgList => false,
            _ => true,
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            Tab::LocalPkgList => false,
            Tab::LocalPkg(pkg_tab) => pkg_tab.force_close,
            Tab::RemotePkg(pkg_tab) => pkg_tab.force_close,
            Tab::RemotePkgList => false,
        }
    }
}

#[derive(Default)]
pub enum Tab {
    #[default]
    LocalPkgList,
    RemotePkgList,
    LocalPkg(PkgTab),
    RemotePkg(PkgTab),
}