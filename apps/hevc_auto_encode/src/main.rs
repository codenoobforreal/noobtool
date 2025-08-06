use clap::Parser;
use ffmpeg_utils::process_hevc_encode;
use std::{io::ErrorKind, path::PathBuf, process};
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;
use utils::{handle_walkdir_error, scan_video_from_path};

#[derive(Parser)]
#[command(about = "Batch HEVC video encoding", long_about = None,next_line_help = true)]
struct Cli {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    inputs: Vec<PathBuf>,
    #[arg(short, long, default_value_t = 1, help = "folder recursive depth")]
    depth: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    let inputs_len = cli.inputs.len();

    println!("[info] enter {inputs_len} paths");

    let videos = scan_videos(cli.inputs, cli.depth);

    if videos.is_empty() {
        eprintln!("[warnning] no video found in all your inputs");
        process::exit(1);
    }

    println!("[info] found {} videos in total", videos.len());

    process_encode_tasks(videos).await;

    process::exit(0);
}

async fn process_encode_tasks(videos: Vec<PathBuf>) {
    let cancel_token = CancellationToken::new();

    for video in videos {
        let display_path = video.display();
        let token_clone = cancel_token.clone();
        let video_clone = video.clone();
        let res = spawn_blocking(move || process_hevc_encode(token_clone, video_clone)).await;

        match res {
            Ok(res_future) => match res_future.await {
                Ok(_) => println!("[info] finish encoding: {display_path}"),
                Err(e) => eprintln!("[error] {e}"),
            },
            Err(e) => {
                eprintln!("[error] {e}");
            }
        }
    }
}

fn scan_videos(inputs: Vec<PathBuf>, depth: usize) -> Vec<PathBuf> {
    let mut videos = Vec::with_capacity(inputs.len());

    for input in inputs {
        let display_path = input.display();
        match scan_video_from_path(&input, depth) {
            Ok((s, f)) => {
                videos.extend(s);
                for error in f {
                    let (error_path, cause) = handle_walkdir_error(error);
                    eprintln!("[warning] access {error_path} failed: {cause}");
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("[warnning] {display_path} not found");
                }
                ErrorKind::PermissionDenied => {
                    eprintln!("[warnning] {display_path} no permission");
                }
                _ => {
                    eprintln!("[warnning] {display_path} unknow error: {e}")
                }
            },
        }
    }

    videos
}
