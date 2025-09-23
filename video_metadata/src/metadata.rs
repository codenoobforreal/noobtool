use crate::{Resolution, ResolutionError};
use std::{
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    path::Path,
    process::Command,
    string::FromUtf8Error,
};
use utils::parse_fraction;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Metadata {
    width: u16,
    height: u16,
    /// 平均帧率
    fps: f32,
    /// 时长，单位秒
    duration: f32,
    /// 文件大小，单位字节
    size: u64,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            width: 1_920,
            height: 1_080,
            fps: Default::default(),
            duration: Default::default(),
            size: Default::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("Ffprobe error: {0}")]
    Ffprobe(String),
    #[error(transparent)]
    ReadStdout(#[from] FromUtf8Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error("no such data: {0}")]
    NoSuchData(String),
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}x{}, {}fps, {}s, {}byte",
            self.width, self.height, self.fps, self.duration, self.size
        )
    }
}

impl Metadata {
    /// 用于测试
    pub fn new(width: u16, height: u16, fps: f32, duration: f32, size: u64) -> Self {
        Self {
            width,
            height,
            fps,
            duration,
            size,
        }
    }

    pub fn retrive(video: &Path) -> Result<Self, MetadataError> {
        // ffprobe -v fatal -select_streams v:0 -show_entries stream=width,height,avg_frame_rate -show_entries format=duration,size -of default=noprint_wrappers=1:nokey=1 input.mp4
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "fatal",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height,avg_frame_rate",
                "-show_entries",
                "format=duration,size",
                "-of",
                "default=noprint_wrappers=1",
            ])
            .arg(video)
            .output()?;

        if !output.status.success() {
            let error_msg = if output.stderr.is_empty() {
                format!("Ffprobe exited with status {}", output.status)
            } else {
                String::from_utf8_lossy(&output.stderr).into_owned()
            };
            return Err(MetadataError::Ffprobe(error_msg));
        }

        let out_str = String::from_utf8(output.stdout)?;

        let mut width = None;
        let mut height = None;
        let mut fps = None;
        let mut duration = None;
        let mut size = None;

        for line in out_str.lines() {
            match line {
                s if s.starts_with("width=") => {
                    width = Some(line.trim_start_matches("width=").parse::<u16>()?)
                }
                s if s.starts_with("height=") => {
                    height = Some(line.trim_start_matches("height=").parse::<u16>()?)
                }
                s if s.starts_with("avg_frame_rate=") => {
                    fps = parse_fraction(line.trim_start_matches("avg_frame_rate="))
                }
                s if s.starts_with("duration=") => {
                    duration = Some(line.trim_start_matches("duration=").parse::<f32>()?)
                }
                s if s.starts_with("size=") => {
                    size = Some(line.trim_start_matches("size=").parse::<u64>()?)
                }
                _ => (),
            };
        }

        let width = width.ok_or_else(|| MetadataError::NoSuchData("width".into()))?;
        let height = height.ok_or_else(|| MetadataError::NoSuchData("height".into()))?;
        let fps = fps.ok_or_else(|| MetadataError::NoSuchData("fps".into()))?;
        let duration = duration.ok_or_else(|| MetadataError::NoSuchData("duration".into()))?;
        let size = size.ok_or_else(|| MetadataError::NoSuchData("size".into()))?;

        Ok(Metadata::new(width, height, fps, duration, size))
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn resolution(&self) -> Result<Resolution, ResolutionError> {
        Resolution::new(self.width, self.height)
    }

    pub fn pixels(&self) -> u32 {
        (self.width as u32) * (self.height as u32)
    }
}
