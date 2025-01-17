mod ui;

use {
    alpm::{Alpm, Package},
    ouroboros::self_referencing,
    ui::UiState,
};

pub struct PacfrontApp {
    pac: PacState,
    ui: UiState,
}

#[self_referencing]
struct PacState {
    alpm: Alpm,
    #[borrows(alpm)]
    db: &'this alpm::Db,
    #[borrows(alpm)]
    #[covariant]
    sync: Vec<&'this alpm::Db>,
    #[borrows(db)]
    #[covariant]
    pkg_list: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    sync_pkg_list: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    filtered_local_pkgs: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    filtered_sync_pkgs: Vec<&'this Package>,
}

impl PacState {
    fn gimme_new() -> anyhow::Result<Self> {
        let alpm = alpm_utils::alpm_with_conf(&alpm_utils::config::Config::new()?)?;
        let mut neu = PacStateBuilder {
            alpm,
            db_builder: |alpm| alpm.localdb(),
            sync_builder: |alpm| alpm.syncdbs().into_iter().collect(),
            pkg_list_builder: |db| db.pkgs().into_iter().collect(),
            sync_pkg_list_builder: |_db| Vec::new(),
            filtered_local_pkgs_builder: |_db| Vec::new(),
            filtered_sync_pkgs_builder: |_db| Vec::new(),
        }
        .build();
        neu.with_mut(|this| {
            *this.filtered_local_pkgs = this.pkg_list.clone();
            *this.sync_pkg_list = this.sync.iter_mut().flat_map(|db| db.pkgs()).collect();
            *this.filtered_sync_pkgs = this.sync_pkg_list.clone();
        });
        Ok(neu)
    }
}

impl PacfrontApp {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            pac: PacState::gimme_new()?,
            ui: UiState::default(),
        })
    }
}

impl eframe::App for PacfrontApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ui::top_panel_ui(self, ctx);
        ui::central_panel_ui(self, ctx);
        ui::process_cmds(self, ctx);
    }
}
