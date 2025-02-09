use crate::shortcuts::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;

// #[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentSettings {
    pub accent_color: [u8; 3],
    pub vsync: bool,
    pub shortcuts: Shortcuts,
}

impl Default for PersistentSettings {
    fn default() -> Self {
        PersistentSettings {
            accent_color: [255, 0, 75],
            vsync: true,
            shortcuts: Shortcuts::default_keys(),
        }
    }
}

impl PersistentSettings {
    pub fn load() -> Result<Self> {
        let local_dir = dirs::data_local_dir().ok_or(anyhow!("Can't get local dir"))?;
        let f = File::open(local_dir.join(".oculante"))?;
        Ok(serde_json::from_reader::<_, PersistentSettings>(f)?)
    }

    pub fn save(&self) -> Result<()> {
        let local_dir = dirs::data_local_dir().ok_or(anyhow!("Can't get local dir"))?;
        let f = File::create(local_dir.join(".oculante"))?;
        Ok(serde_json::to_writer_pretty(f, self)?)
    }
}
