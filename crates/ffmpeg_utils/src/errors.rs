#[derive(Debug, thiserror::Error)]

pub enum EncodeError {
    #[error("failed to parse json: {0}")]
    ParseJson(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Output path: {0}")]
    OutputPath(String),
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
    #[error("Metadata error: {0}")]
    Metadata(#[from] MetadataError),
    #[error("Ffmpeg error: {0}")]
    Ffmpeg(#[from] FfmpegError),
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("failed to parse json: {0}")]
    ParseJson(String),
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
    #[error("Ffprobe error: {0}")]
    Ffprobe(#[from] FfprobeError),
}

#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("failed to parse json: {0}")]
    ParseJson(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Output path: {0}")]
    OutputPath(String),
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
    #[error("Metadata error: {0}")]
    Metadata(#[from] MetadataError),
    #[error("Ffmpeg error: {0}")]
    Ffmpeg(#[from] FfmpegError),
}

#[derive(Debug, thiserror::Error)]
pub enum FfprobeError {
    #[error("Ffprobe error: {0}")]
    Inner(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("Ffmpeg error: {0}")]
    Inner(String),
    #[error("Ffmpeg timeout: {0}")]
    Timeout(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("canceled")]
    Canceled,
    #[error("spawn error: {0}")]
    Spawn(String),
    #[error("kill error: {0}")]
    Kill(String),
    #[error("exit status error: {0}")]
    ExitStatus(String),
}
