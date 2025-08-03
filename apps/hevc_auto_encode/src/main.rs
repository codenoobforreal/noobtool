use ffmpeg_utils::{EncodeError, Encoder};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};
use tokio_util::sync::CancellationToken;
use utils::{find_video_within_folder, is_root_path, is_video_path};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("no path found");
        process::exit(1);
    }

    let mut videos = Vec::with_capacity(args.len());

    for arg in args {
        let path = Path::new(&arg);
        if !is_root_path(path) {
            match fs::symlink_metadata(path).ok() {
                Some(meta) if meta.is_dir() => {
                    videos.extend(find_video_within_folder(path, 3));
                }
                Some(meta) if meta.is_file() && is_video_path(path) => {
                    videos.push(PathBuf::from(arg));
                }
                _ => {}
            }
        }
    }

    if videos.is_empty() {
        eprintln!("no video found");
        process::exit(1);
    }

    if let Err(e) = process_encode_tasks(videos).await {
        eprintln!("{e}")
    }

    println!("process all encode tasks");
    process::exit(0);
}

async fn process_encode_tasks(videos: Vec<PathBuf>) -> Result<(), EncodeError> {
    for video in videos {
        match Encoder::new(video, CancellationToken::new()).await {
            Ok(encoder) => encoder.encode().await?,
            Err(e) => eprintln!("{e}"),
        }
    }

    Ok(())
}
