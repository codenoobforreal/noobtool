use ffmpeg_progress_monitor::ProgressMonitorError;
use std::io;
use video_metadata::ResolutionError;

#[derive(Debug, thiserror::Error)]
pub enum EncoderError {
    #[error("failed to get stem portion of {0}")]
    FileStem(String),
    #[error(transparent)]
    Resolution(#[from] ResolutionError),
    #[error(transparent)]
    ProgressMonitor(#[from] ProgressMonitorError),
    #[error(transparent)]
    TO(#[from] io::Error),
    #[error("failed to get stderr")]
    TakeStd,
    #[error("FFmpeg exited with status {0}")]
    FfmpegExit(String),
}

pub(crate) type EncodeResult<T> = Result<T, EncoderError>;
