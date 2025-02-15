use anyhow::Result;
use serde::{Deserialize, Serialize};

use bitvec_helpers::{bitslice_reader::BitSliceReader, bitstream_io_writer::BitstreamIoWriter};

use super::edit_config::EditMdcvMetadata;

const D65_WHITEPOINT: [u16; 2] = [15635, 16450];
const MDL_FACTOR: f32 = 10_000.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MdcvMetadata {
    pub primaries: MasteringDisplayPrimaries,

    /// in units of 0.0001 nits
    /// 1000 nits = 10000000, 0.0001 = 1
    pub max_display_mastering_luminance: u32,
    pub min_display_mastering_luminance: u32,
}

/// Values in units of 0.00002
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MasteringDisplayPrimaries {
    pub display_primaries_x: [u16; 3],
    pub display_primaries_y: [u16; 3],
    pub white_point: [u16; 2],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MdcvPrimariesPreset {
    #[serde(alias = "bt.709")]
    #[serde(alias = "709")]
    BT709,
    #[serde(alias = "display-p3")]
    #[serde(alias = "p3-d65")]
    DisplayP3,
    #[serde(alias = "bt.2020")]
    #[serde(alias = "2020")]
    BT2020,
}

impl MdcvMetadata {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut reader = BitSliceReader::new(data);

        let mut display_primaries_x = [0; 3];
        let mut display_primaries_y = [0; 3];

        for c in 0..3 {
            display_primaries_x[c] = reader.get_n(16)?;
            display_primaries_y[c] = reader.get_n(16)?;
        }

        let white_point = [reader.get_n(16)?, reader.get_n(16)?];

        let primaries = MasteringDisplayPrimaries {
            display_primaries_x,
            display_primaries_y,
            white_point,
        };

        let max_display_mastering_luminance = reader.get_n(32)?;
        let min_display_mastering_luminance = reader.get_n(32)?;
        Ok(Self {
            primaries,
            max_display_mastering_luminance,
            min_display_mastering_luminance,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = BitstreamIoWriter::with_capacity(24);

        for c in 0..3 {
            writer.write_n(&self.primaries.display_primaries_x[c], 16)?;
            writer.write_n(&self.primaries.display_primaries_y[c], 16)?;
        }

        for v in &self.primaries.white_point {
            writer.write_n(v, 16)?;
        }

        writer.write_n(&self.max_display_mastering_luminance, 32)?;
        writer.write_n(&self.min_display_mastering_luminance, 32)?;

        Ok(writer.into_inner())
    }

    pub fn copy(mut self, src: &EditMdcvMetadata) -> Self {
        if let Some(new_primaries) = &src.primaries {
            self.primaries = new_primaries.clone();
        } else if let Some(new_preset) = &src.preset {
            self.primaries = new_preset.primaries();
        }

        if let Some(min_mdl) = src.min_display_mastering_luminance {
            self.min_display_mastering_luminance = (min_mdl * MDL_FACTOR).round() as u32;
        }

        if let Some(max_mdl) = src.max_display_mastering_luminance {
            self.max_display_mastering_luminance = (max_mdl * MDL_FACTOR).round() as u32;
        }

        self
    }
}

impl MasteringDisplayPrimaries {
    pub const fn bt709() -> Self {
        MasteringDisplayPrimaries {
            display_primaries_x: [32000, 15000, 7500],
            display_primaries_y: [16500, 30000, 3000],
            white_point: D65_WHITEPOINT,
        }
    }

    pub const fn displayp3() -> Self {
        MasteringDisplayPrimaries {
            display_primaries_x: [34000, 13250, 7500],
            display_primaries_y: [16000, 34500, 3000],
            white_point: D65_WHITEPOINT,
        }
    }

    pub const fn bt2020() -> Self {
        MasteringDisplayPrimaries {
            display_primaries_x: [35400, 8500, 6550],
            display_primaries_y: [14600, 39850, 2300],
            white_point: D65_WHITEPOINT,
        }
    }
}

impl MdcvPrimariesPreset {
    pub const fn primaries(&self) -> MasteringDisplayPrimaries {
        match self {
            Self::BT709 => MasteringDisplayPrimaries::bt709(),
            Self::DisplayP3 => MasteringDisplayPrimaries::displayp3(),
            Self::BT2020 => MasteringDisplayPrimaries::bt2020(),
        }
    }
}
