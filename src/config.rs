use {
    anyhow::Context,
    ron::ser::PrettyConfig,
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

pub fn cfg_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|cfg_dir| cfg_dir.join("pacfront"))
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub color_theme: Option<[Rgb; 12]>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = cfg_dir()
            .context("Couldn't get config path")?
            .join("config.ron");
        let string = std::fs::read_to_string(path)?;
        let cfg: Self = ron::from_str(&string)?;
        Ok(cfg)
    }
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config: {e}. Using default.");
                Self::default()
            }
        }
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let out = ron::ser::to_string_pretty(self, PrettyConfig::default())?;
        let dir = cfg_dir().context("Couldn't get config path")?;
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("config.ron"), out.as_bytes())?;
        Ok(())
    }
}

type Rgb = [u8; 3];
