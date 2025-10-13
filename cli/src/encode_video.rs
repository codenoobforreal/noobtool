use clap::{Args, value_parser};
use std::path::PathBuf;
// use video_encoder::Preset;
use video_metadata::Resolution;

#[derive(Args, Debug)]
#[command(about = "batch video encoding")]
pub struct EncodeVideoArgs {
    #[arg(short, long, long_help = "input video or folder")]
    pub inputs: Vec<PathBuf>,
    // #[arg(short, long, default_value_t = Preset::Medium,long_help = "video encoding preset")]
    // pub preset: Preset,
    #[arg(short, long, default_value_t = Resolution::default(),long_help = "limit resolution")]
    pub resolution: Resolution,
    #[arg(short, long, default_value_t = 24,value_parser = value_parser!(u8).range(1..))]
    pub fps: u8,
    #[arg(short, long, default_value_t = 1,value_parser = value_parser!(u8).range(1..), help = "folder recursive depth")]
    pub depth: u8,
}
