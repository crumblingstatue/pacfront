mod ui;

use {
    alpm::{Alpm, Package, SigLevel},
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
    filtered_local_pkgs: Vec<&'this Package>,
}

impl PacState {
    fn gimme_new() -> anyhow::Result<Self> {
        let alpm = Alpm::new2("/", "/var/lib/pacman")?;
        // Enumerate sync DBs
        //
        // TODO: Source repos from /etc/pacman.conf
        // TODO: Can be enumerated from /var/lib/pacman/sync, but not all
        //       of it might be enabled, hence the need for pacman.conf
        // TODO: What is SigLevel?
        // TODO: Better error handling/logging
        for entry in std::fs::read_dir("/var/lib/pacman/sync")?.flatten() {
            if let Some(stem) = entry.path().file_stem() {
                if let Some(name) = stem.to_str() {
                    alpm.register_syncdb(name, SigLevel::NONE)?;
                }
            }
        }
        let mut neu = PacStateBuilder {
            alpm,
            db_builder: |alpm| alpm.localdb(),
            sync_builder: |alpm| alpm.syncdbs().into_iter().collect(),
            pkg_list_builder: |db| db.pkgs().into_iter().collect(),
            filtered_local_pkgs_builder: |_db| Vec::new(),
        }
        .build();
        neu.with_mut(|this| {
            *this.filtered_local_pkgs = this.pkg_list.clone();
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
