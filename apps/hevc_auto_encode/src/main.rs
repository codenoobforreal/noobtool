use clap::Parser;
use ffmpeg_utils::process_hevc_encode;
use std::{fs, path::PathBuf, process};
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;
use utils::{find_video_within_folder, is_video_path};

#[derive(Parser)]
#[command(about = "Batch HEVC video encoding", long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    inputs: Vec<String>,
    #[arg(short, long, default_value_t = 1, help = "folder recursive depth")]
    depth: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    let mut videos = Vec::with_capacity(cli.inputs.len());

    for input in cli.inputs {
        match fs::symlink_metadata(&input).ok() {
            Some(meta) if meta.is_dir() => {
                videos.extend(find_video_within_folder(input, cli.depth));
            }
            Some(meta) if meta.is_file() && is_video_path(&input) => {
                videos.push(PathBuf::from(input));
            }
            _ => {}
        }
    }

    if videos.is_empty() {
        eprintln!("no video found in all your inputs");
        process::exit(1);
    }

    process_encode_tasks(videos).await;

    println!("complete all encoding tasks");
    process::exit(0);
}

async fn process_encode_tasks(videos: Vec<PathBuf>) {
    let cancel_token = CancellationToken::new();

    for video in videos {
        let token_clone = cancel_token.clone();
        let video_clone = video.clone();
        let res = spawn_blocking(move || process_hevc_encode(token_clone, video_clone)).await;

        match res {
            Ok(res_future) => match res_future.await {
                Ok(_) => println!("{video:?}"),
                Err(e) => eprintln!("{e}"),
            },
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }
}
