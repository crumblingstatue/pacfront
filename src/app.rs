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
    local_pkg_list: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    remote_pkg_list: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    filt_local_pkg_list: Vec<&'this Package>,
    #[borrows(db)]
    #[covariant]
    filt_remote_pkg_list: Vec<&'this Package>,
}

impl PacState {
    fn gimme_new() -> anyhow::Result<Self> {
        let alpm = alpm_utils::alpm_with_conf(&alpm_utils::config::Config::new()?)?;
        let mut neu = PacStateBuilder {
            alpm,
            db_builder: |alpm| alpm.localdb(),
            sync_builder: |alpm| alpm.syncdbs().into_iter().collect(),
            local_pkg_list_builder: |db| db.pkgs().into_iter().collect(),
            remote_pkg_list_builder: |_db| Vec::new(),
            filt_local_pkg_list_builder: |_db| Vec::new(),
            filt_remote_pkg_list_builder: |_db| Vec::new(),
        }
        .build();
        neu.with_mut(|this| {
            *this.filt_local_pkg_list = this.local_pkg_list.clone();
            *this.remote_pkg_list = this.sync.iter_mut().flat_map(|db| db.pkgs()).collect();
            *this.filt_remote_pkg_list = this.remote_pkg_list.clone();
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
        ui::cmd::process_cmds(self, ctx);
    }
}
