use ffmpeg_progress_monitor::ProgressMonitorError;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("failed to get stem portion of {0}")]
    FileStem(String),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    ProgressMonitor(#[from] ProgressMonitorError),
    #[error("failed to get stderr")]
    TakeStd,
    #[error("FFmpeg exited with status {0}")]
    FfmpegExit(String),
    #[error("delimiter x not found")]
    NoDelimiterX,
    #[error("row is missing")]
    MissingRow,
    #[error("col is missing")]
    MissingCol,
    #[error("error parsing {0}")]
    Parse(String),
}

pub(crate) type ThumbnailResult<T> = Result<T, ThumbnailError>;
