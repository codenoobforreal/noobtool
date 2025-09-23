use clap::{Args, value_parser};
use std::path::PathBuf;

#[derive(Args)]
#[command(about = "batch video thumbnail generation")]
pub struct GenerateVideoThumbnailArgs {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    inputs: Vec<PathBuf>,
    #[arg(short, long, default_value_t = 1,value_parser = value_parser!(u8).range(1..), help = "folder recursive depth")]
    depth: u8,
}
