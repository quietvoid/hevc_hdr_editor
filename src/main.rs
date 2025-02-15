use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};

mod cll_metadata;
mod edit_config;
mod mdcv_metadata;
mod processor;
mod utils;
use processor::Processor;

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"), about = "Utility to losslessly edit HDR metadata in HEVC files", author = "quietvoid", version = env!("CARGO_PKG_VERSION"))]
pub struct Opt {
    #[arg(
        id = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Sets the output JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        short = 'c',
        help = "Sets the edit JSON config file to use",
        value_hint = ValueHint::FilePath
    )]
    config: PathBuf,
}
fn main() -> Result<()> {
    let opt = Opt::parse();
    Processor::execute(opt)
}
