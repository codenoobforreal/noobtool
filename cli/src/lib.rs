mod encode_video;
mod generate_video_thumbail;

use clap::{Parser, Subcommand};
pub use encode_video::EncodeVideoArgs;
pub use generate_video_thumbail::GenerateVideoThumbnailArgs;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    EncodeVideo(EncodeVideoArgs),
    GenerateVideoThumbnail(GenerateVideoThumbnailArgs),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
