use anyhow::Result;
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitstream_io_writer::BitstreamIoWriter};
use serde::{Deserialize, Serialize};

use super::edit_config::EditCllMetadata;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CllMetadata {
    pub max_content_light_level: u16,
    pub max_average_light_level: u16,
}

impl CllMetadata {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut reader = BitSliceReader::new(data);

        Ok(Self {
            max_content_light_level: reader.get_n(16)?,
            max_average_light_level: reader.get_n(16)?,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = BitstreamIoWriter::with_capacity(4);

        writer.write_n(&self.max_content_light_level, 16)?;
        writer.write_n(&self.max_average_light_level, 16)?;

        Ok(writer.into_inner())
    }

    pub fn copy(mut self, src: &EditCllMetadata) -> Self {
        if let Some(max_cll) = src.max_content_light_level {
            self.max_content_light_level = max_cll;
        }

        if let Some(max_fall) = src.max_average_light_level {
            self.max_average_light_level = max_fall;
        }

        self
    }
}
