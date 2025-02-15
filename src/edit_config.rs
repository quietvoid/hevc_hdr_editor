use std::fs::File;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::mdcv_metadata::{MasteringDisplayPrimaries, MdcvPrimariesPreset};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditConfig {
    pub mdcv: Option<EditMdcvMetadata>,
    pub cll: Option<EditCllMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMdcvMetadata {
    /// Existing preset display primaries (BT.709, Display-P3 or BT.2020)
    pub preset: Option<MdcvPrimariesPreset>,

    /// `Some` to edit, `None` to leave untouched
    pub primaries: Option<MasteringDisplayPrimaries>,

    /// In nits
    /// Example: min: 0.001 nits, max: 1000 nits
    pub max_display_mastering_luminance: Option<f32>,
    pub min_display_mastering_luminance: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditCllMetadata {
    pub max_content_light_level: Option<u16>,
    pub max_average_light_level: Option<u16>,
}

impl EditConfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json_file = File::open(path)?;
        let config: EditConfig = serde_json::from_reader(&json_file)?;

        Ok(config)
    }

    pub fn setup(&mut self) -> Result<()> {
        if self.mdcv.is_none() && self.cll.is_none() {
            bail!("One of either MDCV or CLL metadata must be present");
        }

        Ok(())
    }
}
