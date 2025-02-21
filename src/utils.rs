use std::path::PathBuf;
use std::{fs::File, path::Path};

use anyhow::{Result, bail, ensure};
use bitvec_helpers::bitstream_io_writer::BitstreamIoWriter;
use hevc_parser::hevc::{NAL_SEI_PREFIX, SeiMessage};
use hevc_parser::utils::add_start_code_emulation_prevention_3_byte;
use indicatif::{ProgressBar, ProgressStyle};

use hevc_parser::io::IoFormat;

use super::processor::EditedSei;

pub fn initialize_progress_bar<P: AsRef<Path>>(format: &IoFormat, input: P) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let IoFormat::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan} {percent}%")?,
        );
    }

    Ok(pb)
}

pub fn input_from_either(in1: Option<PathBuf>, in2: Option<PathBuf>) -> Result<PathBuf> {
    match in1 {
        Some(in1) => Ok(in1),
        None => match in2 {
            Some(in2) => Ok(in2),
            None => bail!("No input file provided. See `hevc_hdr_editor --help`"),
        },
    }
}

pub fn sei_message_data<'a>(msg: &SeiMessage, sei_payload: &'a [u8]) -> &'a [u8] {
    let start = msg.payload_offset;
    let end = start + msg.payload_size;

    &sei_payload[start..end]
}

pub fn encode_payload_to_sei_prefix(payload_type: u8, payload: &[u8]) -> Result<Vec<u8>> {
    // Write NALU SEI_PREFIX header
    let mut header_writer = BitstreamIoWriter::with_capacity(payload.len());

    header_writer.write(false)?; // forbidden_zero_bit

    header_writer.write_n(&NAL_SEI_PREFIX, 6)?; // nal_type
    header_writer.write_n(&0_u8, 6)?; // nuh_layer_id
    header_writer.write_n(&1_u8, 3)?; // nuh_temporal_id_plus1

    header_writer.write_n(&payload_type, 8)?;

    // FIXME: This should probably be 1024 but not sure how to write a longer header
    ensure!(
        payload.len() <= 255,
        "Payload too large: {} bytes",
        payload.len()
    );

    header_writer.write_n(&(payload.len() as u64), 8)?;

    let mut data = header_writer.into_inner();
    data.extend_from_slice(payload);

    data.push(0x80);

    add_start_code_emulation_prevention_3_byte(&mut data);

    Ok(data)
}

pub fn encode_edited_sei_to_nal(edited_sei: EditedSei) -> Result<Vec<u8>> {
    encode_payload_to_sei_prefix(edited_sei.payload_type(), &edited_sei.encode_payload()?)
}
