use crate::errors::{FfprobeError, MetadataError, ProcessError};
use serde::{Deserialize, Deserializer};
use std::{
    ffi::{OsStr, OsString},
    process::Stdio,
};
use tokio::{io::AsyncReadExt, process::Command, select};
use tokio_util::sync::CancellationToken;

pub async fn get_metadata(
    video_path: &OsStr,
    cancel_token: CancellationToken,
) -> Result<MetadataJsonRoot, MetadataError> {
    // ffprobe -v error -select_streams v:0 -show_format -show_streams -of json output.mp4
    let mut child_process = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(OsString::from(video_path))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ProcessError::Spawn(e.to_string()))?;

    let stdout = child_process.stdout.take();
    let stderr = child_process.stderr.take();
    let mut out_str = String::new();
    let mut err_str = String::new();

    select! {
        _ = cancel_token.cancelled() => {
            match child_process.kill().await {
              Ok(_) => Err(MetadataError::Process(ProcessError::Canceled)),
              Err(e) => Err(MetadataError::Process(ProcessError::Kill(e.to_string()))),
            }
        }
        status = child_process.wait() => {
            match status {
              Ok(status) if status.success() => {
                if let Some(mut pipe) = stdout {
                  pipe.read_to_string(&mut out_str).await.ok();
                }
                let json = serde_json::from_str(&out_str)
                .map_err(|e| MetadataError::ParseJson(e.to_string()))?;
                Ok(json)
              }
              Ok(_) => {
                if let Some(mut pipe) = stderr {
                  pipe.read_to_string(&mut err_str).await.ok();
                }
                Err(MetadataError::Ffprobe(FfprobeError::Inner(err_str)))
              }
              Err(e) => Err(MetadataError::Process(ProcessError::ExitStatus(e.to_string()))),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MetadataJsonRoot {
    pub streams: Vec<Streams>,
    pub format: Format,
}

impl MetadataJsonRoot {
    pub fn first_stream(&self) -> Option<&Streams> {
        self.streams.first()
    }
    pub fn title(&self) -> String {
        self.streams
            .first()
            .map(|s| s.tags.title.as_str())
            .filter(|s| !s.is_empty()) // filter default string value
            .unwrap_or(self.format.tags.title.as_str())
            .to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct Streams {
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub width: f32,
    pub height: f32,
    pub coded_width: f32,
    pub coded_height: f32,
    #[serde(default)]
    pub sample_aspect_ratio: String,
    #[serde(default)]
    pub display_aspect_ratio: String,
    pub pix_fmt: String,
    #[serde(default)]
    pub color_space: String,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    pub start_time: F32FromStr,
    #[serde(default)]
    pub tags: StreamsTags,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct StreamsTags {
    pub title: String,
    #[serde(rename = "HANDLER_NAME")]
    pub handler_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub filename: String,
    pub nb_streams: usize,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: String,
    pub duration: F32FromStr,
    pub size: F32FromStr,
    pub bit_rate: F32FromStr,
    #[serde(default)]
    pub tags: FormatTags,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct FormatTags {
    pub title: String,
    #[serde(rename = "ENCODER")]
    pub encoder: String,
}

#[derive(Debug)]
pub struct F32FromStr(pub f32);

impl<'de> Deserialize<'de> for F32FromStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let val = s.parse().map_err(serde::de::Error::custom)?;
        Ok(F32FromStr(val))
    }
}
