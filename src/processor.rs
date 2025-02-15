use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use hevc_parser::utils::clear_start_code_emulation_prevention_3_byte;
use indicatif::ProgressBar;

use hevc_parser::hevc::{NALUnit, SeiMessage, NAL_SEI_PREFIX};
use hevc_parser::io::processor::{HevcProcessor, HevcProcessorOpts};
use hevc_parser::io::{IoFormat, IoProcessor, StartCodePreset};
use hevc_parser::HevcParser;
use num_enum::TryFromPrimitive;

use crate::utils::{encode_edited_sei_to_nal, encode_payload_to_sei_prefix};

use super::cll_metadata::CllMetadata;
use super::mdcv_metadata::MdcvMetadata;
use super::utils::sei_message_data;
use super::{edit_config::EditConfig, Opt};

pub struct Processor {
    input: PathBuf,
    config: EditConfig,

    progress_bar: ProgressBar,
    writer: BufWriter<File>,
}

#[derive(TryFromPrimitive, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
enum SeiPayloadType {
    MasteringDisplayColourVolume = 137,
    ContentLightLevel = 144,
}

pub enum EditedSei<'a> {
    None(&'a SeiMessage),
    Mdcv((&'a SeiMessage, MdcvMetadata)),
    Cll((&'a SeiMessage, CllMetadata)),
}

impl Processor {
    pub fn execute(opt: Opt) -> Result<()> {
        let mut config = EditConfig::from_path(&opt.config)?;
        config.setup()?;

        let Opt {
            input,
            input_pos,
            output,
            ..
        } = opt;

        let input = crate::utils::input_from_either(input, input_pos)?;

        let format = hevc_parser::io::format_from_path(&input)?;

        let hevc_out = match output {
            Some(path) => path,
            None => PathBuf::from("hdr_edited_output.hevc"),
        };

        let pb = crate::utils::initialize_progress_bar(&format, &input)?;

        let mut processor = Self {
            input,
            config,
            progress_bar: pb,
            writer: BufWriter::with_capacity(
                100_000,
                File::create(&hevc_out).expect("Can't create file"),
            ),
        };

        processor.process_input(&format)
    }

    pub fn process_input(&mut self, format: &IoFormat) -> Result<()> {
        let chunk_size = 100_000;

        let processor_opts = HevcProcessorOpts {
            parse_nals: false,
            ..Default::default()
        };
        let mut processor = HevcProcessor::new(format.clone(), processor_opts, chunk_size);

        let file_path = if let IoFormat::RawStdin = format {
            None
        } else {
            Some(self.input.clone())
        };

        processor.process_file(self, file_path)
    }

    fn get_edited_sei_for_message<'a>(
        sei_payload: &[u8],
        msg: &'a SeiMessage,
        config: &EditConfig,
    ) -> Result<EditedSei<'a>> {
        let mut ret = Ok(EditedSei::None(msg));

        let payload_type = SeiPayloadType::try_from(msg.payload_type).ok();
        if payload_type.is_none() {
            return ret;
        }

        let payload_type = payload_type.unwrap();
        let data = sei_message_data(msg, sei_payload);

        match payload_type {
            SeiPayloadType::MasteringDisplayColourVolume => {
                if let Some(new_mdcv) = config.mdcv.as_ref() {
                    ret = MdcvMetadata::parse(data)
                        .map(|meta| EditedSei::Mdcv((msg, meta.copy(new_mdcv))));
                }
            }
            SeiPayloadType::ContentLightLevel => {
                if let Some(new_cll) = config.cll.as_ref() {
                    ret = CllMetadata::parse(data)
                        .map(|meta| EditedSei::Cll((msg, meta.copy(new_cll))));
                }
            }
        };

        ret
    }
}

impl IoProcessor for Processor {
    fn input(&self) -> &std::path::PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        for nal in nals {
            let nal_data = &chunk[nal.start..nal.end];

            if nal.nal_type == NAL_SEI_PREFIX {
                let sei_payload = clear_start_code_emulation_prevention_3_byte(nal_data);
                let messages = SeiMessage::parse_sei_rbsp(&sei_payload)?;

                if !messages
                    .iter()
                    .any(|e| SeiPayloadType::try_from(e.payload_type).is_ok())
                {
                    // No message that can be edited, rewrite NAL
                    NALUnit::write_with_preset(
                        &mut self.writer,
                        nal_data,
                        StartCodePreset::Four,
                        nal.nal_type,
                        false,
                    )?;

                    continue;
                }

                let edited_seis = messages
                    .iter()
                    .map(|msg| Self::get_edited_sei_for_message(&sei_payload, msg, &self.config));

                if messages.len() > 1 {
                    let mut new_nals = Vec::with_capacity(messages.len());

                    // Split all messages into separate NALs, even if they're not edited
                    for edited_res in edited_seis {
                        let edited_sei = edited_res?;

                        let nal = match edited_sei {
                            EditedSei::None(msg) => encode_payload_to_sei_prefix(
                                msg.payload_type,
                                sei_message_data(msg, &sei_payload),
                            ),
                            EditedSei::Mdcv(_) | EditedSei::Cll(_) => {
                                encode_edited_sei_to_nal(edited_sei)
                            }
                        }?;

                        new_nals.push(nal);
                    }

                    for data in new_nals {
                        NALUnit::write_with_preset(
                            &mut self.writer,
                            &data,
                            StartCodePreset::Four,
                            nal.nal_type,
                            false,
                        )?;
                    }
                } else if let Some(edited_res) = edited_seis.last() {
                    let edited_sei = edited_res?;
                    let final_data = encode_edited_sei_to_nal(edited_sei)?;

                    NALUnit::write_with_preset(
                        &mut self.writer,
                        &final_data,
                        StartCodePreset::Four,
                        nal.nal_type,
                        false,
                    )?;
                }
            } else {
                NALUnit::write_with_preset(
                    &mut self.writer,
                    nal_data,
                    StartCodePreset::Four,
                    nal.nal_type,
                    false,
                )?;
            }
        }

        Ok(())
    }

    fn finalize(&mut self, _parser: &HevcParser) -> Result<()> {
        self.progress_bar.finish_and_clear();
        self.writer.flush()?;

        Ok(())
    }
}

impl EditedSei<'_> {
    pub const fn payload_type(&self) -> u8 {
        match self {
            Self::Mdcv(_) => SeiPayloadType::MasteringDisplayColourVolume as u8,
            Self::Cll(_) => SeiPayloadType::ContentLightLevel as u8,
            _ => unreachable!(),
        }
    }

    pub fn encode_payload(&self) -> Result<Vec<u8>> {
        match self {
            Self::Mdcv((_, meta)) => meta.encode(),
            Self::Cll((_, meta)) => meta.encode(),
            _ => unreachable!(),
        }
    }
}
