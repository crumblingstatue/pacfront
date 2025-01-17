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
        let mut neu = PacStateBuilder {
            alpm,
            db_builder: |this| this.localdb(),
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
