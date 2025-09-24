use clap::{Args, value_parser};
use std::path::PathBuf;
use video_thumbnail::Grid;

#[derive(Args)]
#[command(about = "batch video thumbnail generation")]
pub struct GenerateVideoThumbnailArgs {
    #[arg(short, long, long_help = "input video or folder")]
    pub inputs: Vec<PathBuf>,

    #[arg(short, long, default_value_t = Grid::default(),long_help = "grid layout: rowxcol")]
    pub grid: Grid,

    #[arg(
        short,
        long,
        default_value_t = 200,
        value_parser = value_parser!(u16).range(1..),
        long_help = "base dimension of grid item"
    )]
    pub base: u16,

    #[arg(short, long, default_value_t = 1,value_parser = value_parser!(u8).range(1..), help = "folder recursive depth")]
    pub depth: u8,
}
