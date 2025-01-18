mod ui;

use {
    crate::config::Config,
    alpm::{Alpm, Package},
    egui_colors::{Colorix, tokens::ThemeColor},
    ouroboros::self_referencing,
    ui::UiState,
};

pub struct PacfrontApp {
    pac: PacState,
    ui: UiState,
    cfg: Config,
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
            cfg: Config::load_or_default(),
        })
    }
    pub fn sync_from_config(&mut self, egui_ctx: &eframe::egui::Context) {
        if let Some(color_theme) = &self.cfg.color_theme {
            self.ui.shared.colorix =
                Some(Colorix::init(egui_ctx, color_theme.map(ThemeColor::Custom)))
        }
    }
    fn sync_to_config(&mut self) {
        self.cfg.color_theme = self
            .ui
            .shared
            .colorix
            .as_ref()
            .map(|colorix| colorix.theme().map(|theme| theme.rgb()));
    }
}

impl eframe::App for PacfrontApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ui::top_panel_ui(self, ctx);
        ui::central_panel_ui(self, ctx);
        ui::cmd::process_cmds(self, ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sync_to_config();
        if let Err(e) = self.cfg.save() {
            eprintln!("Failed to save config: {e}");
        }
    }
}
