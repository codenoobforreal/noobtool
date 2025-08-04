use ffmpeg_utils::process_hevc_encode;
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

    process_encode_tasks(videos).await;

    println!("complete all hevc encode tasks");
    process::exit(0);
}

async fn process_encode_tasks(videos: Vec<PathBuf>) {
    for video in videos {
        match process_hevc_encode(CancellationToken::new(), video.clone()).await {
            Ok(_) => println!("finish video ({video:?}) hevc encoding"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
