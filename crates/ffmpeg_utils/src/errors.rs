use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfprobeError {
    #[error("Ffprobe error: {0}")]
    Inner(String),
}

#[derive(Debug, Error)]
pub enum FfmpegError {
    #[error("Ffmpeg error: {0}")]
    Inner(String),
    #[error("Ffmpeg execute timeout: {0}")]
    Timeout(String),
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Process canceled")]
    Canceled,
    #[error("Process spawn error: {0}")]
    Spawn(#[source] io::Error),
    #[error("Process kill error: {0}")]
    Kill(#[source] io::Error),
    #[error("Process exit status error: {0}")]
    ExitStatus(#[source] io::Error),
}
