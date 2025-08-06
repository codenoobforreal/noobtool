use clap::Parser;
use ffmpeg_utils::process_thumbnail_generate;
use std::{io::ErrorKind, path::PathBuf, process};
use tokio::{
    task::spawn_blocking,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use utils::{handle_walkdir_error, scan_video_from_path};

#[derive(Parser)]
#[command(about = "Batch video thumbnail generation", long_about = None, next_line_help = true)]
pub struct Cli {
    #[arg(
        short,
        long,
        num_args = 1..,
        long_help = "video inputs, can be file and folder path seprated by space"
    )]
    pub inputs: Vec<PathBuf>,
    #[arg(short, long, default_value_t = 1, help = "folder recursive depth")]
    pub depth: usize,
    #[arg(
        short,
        long,
        default_value_t = 180,
        help = "timeout for the thumbnail task"
    )]
    pub timeout: usize,
    #[arg(
        short,
        long,
        default_value_t = 200.0,
        help = "base thumbnail grid size"
    )]
    pub length: f32,
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

    process_thumbnail_tasks(videos, Duration::from_secs(cli.timeout as u64), cli.length).await;

    process::exit(0);
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

async fn process_thumbnail_tasks(videos: Vec<PathBuf>, timeout: Duration, base_dim: f32) {
    let cancel_token = CancellationToken::new();

    let whole_duration = Instant::now();

    let mut success_count = 0;
    let mut fail_count = 0;

    for video in videos {
        let token_clone = cancel_token.clone();
        let video_clone = video.clone();
        let task_duration = Instant::now();
        let res = spawn_blocking(move || {
            process_thumbnail_generate(token_clone, timeout, video_clone, base_dim)
        })
        .await;

        match res {
            Ok(res_future) => match res_future.await {
                Ok(_) => {
                    success_count += 1;
                    println!(
                        "[info] {:.1?} on {:?}",
                        task_duration.elapsed(),
                        video.display()
                    );
                }
                Err(e) => {
                    fail_count += 1;
                    eprintln!("[error] {e}")
                }
            },
            Err(e) => eprintln!("[error] {e}"),
        }
    }

    println!(
        "[info] {success_count} tasks success, {fail_count} tasks fail. duration: {:.1?}",
        whole_duration.elapsed()
    )
}
