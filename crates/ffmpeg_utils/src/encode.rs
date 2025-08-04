use crate::{
    errors::{FfmpegError, ProcessError},
    get_metadata,
    metadata::MetadataError,
};
use chrono::Local;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{io::AsyncReadExt, process::Command, select};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Json miss field: {0}")]
    MissingField(String),
    #[error("Generate output of {input} failed")]
    OutputPath { input: String },
    #[error(transparent)]
    Process(#[from] ProcessError),
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    #[error(transparent)]
    Ffmpeg(#[from] FfmpegError),
}

pub async fn process_hevc_encode(
    cancel_token: CancellationToken,
    input: PathBuf,
) -> Result<(), EncodeError> {
    let token_clone = cancel_token.clone();
    select! {
        _ = token_clone.cancelled() => Err(ProcessError::Canceled)?,
        res = async {
          let config = Config::init(input)?;
          let info = get_required_info(config.input.as_os_str(), cancel_token.clone()).await?;
          let encoder = Encoder::new(config,cancel_token,info);
          encoder.run().await
        } => res,
    }
}

struct Config {
    input: PathBuf,
    output: PathBuf,
}

impl Config {
    fn init(input: PathBuf) -> Result<Self, EncodeError> {
        let output = generate_output_path(&input)?;
        Ok(Config { input, output })
    }
}

struct Encoder {
    config: Config,
    cancel_token: CancellationToken,
    info: RequiredInfo,
}

impl Encoder {
    fn new(config: Config, cancel_token: CancellationToken, info: RequiredInfo) -> Self {
        Self {
            config,
            cancel_token,
            info,
        }
    }

    // ffmpeg -v error -progress pipe:2 -i input.mp4 -c:v libx265 -x265-params log-level=error -crf 20 -f mp4 -c:a copy output.mp4
    fn build_command_args(&self) -> Vec<OsString> {
        let mut args = vec![
            OsString::from("-v"),
            OsString::from("error"),
            OsString::from("-progress"),
            OsString::from("pipe:2"),
            OsString::from("-i"),
            OsString::from(&self.config.input),
            OsString::from("-c:v"),
            OsString::from("libx265"),
            OsString::from("-x265-params"),
            OsString::from("log-level=error"),
            OsString::from("-crf"),
            OsString::from(self.info.crf.to_string()),
            OsString::from("-f"),
            OsString::from("mp4"),
            OsString::from("-c:a"),
            OsString::from("copy"),
            OsString::from(&self.config.output),
        ];

        if self.info.scale_width.is_some() {
            args.splice(
                10..10,
                [OsString::from("-vf"), OsString::from("scale=1920:-2")],
            );
        }

        if self.info.scale_height.is_some() {
            args.splice(
                10..10,
                [OsString::from("-vf"), OsString::from("scale=-2:1920")],
            );
        }

        args
    }

    async fn run(&self) -> Result<(), EncodeError> {
        let args = self.build_command_args();

        let mut child = Command::new("ffmpeg")
            .args(args)
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(ProcessError::Spawn)?;

        let stderr = child.stderr.take();
        let mut error_str = String::new();

        select! {
            _ = self.cancel_token.cancelled() => {
                match child.kill().await {
                    Ok(()) => Err(ProcessError::Canceled)?,
                    Err(e) => Err(ProcessError::Kill(e))?,
                }
            }
            res = child.wait() => {
                if let Some(mut pipe) = stderr {
                    pipe.read_to_string(&mut error_str).await.ok();
                }
                match res {
                    Ok(status) if status.success() => Ok(()),
                    Ok(_) => Err(FfmpegError::Inner(error_str))?,
                    Err(e) => Err(ProcessError::ExitStatus(e))?,
                }
            }
        }
    }
}

struct RequiredInfo {
    crf: usize,
    scale_width: Option<usize>,
    scale_height: Option<usize>,
}

impl RequiredInfo {
    fn new(crf: usize, scale_width: Option<usize>, scale_height: Option<usize>) -> Self {
        RequiredInfo {
            crf,
            scale_width,
            scale_height,
        }
    }
}

async fn get_required_info(
    video: &OsStr,
    cancel_token: CancellationToken,
) -> Result<RequiredInfo, EncodeError> {
    let display_path = video.display();
    let metadata = get_metadata(video, cancel_token).await?;
    let video_stream = metadata.first_stream().ok_or_else(|| {
        EncodeError::MissingField(format!("{display_path} failed to get stream info"))
    })?;

    let pixels = video_stream.width * video_stream.height;

    // https://handbrake.fr/docs/en/1.9.0/workflow/adjust-quality.html
    if pixels >= 2073600.0 {
        if video_stream.width >= video_stream.height {
            Ok(RequiredInfo::new(20, Some(1920), None))
        } else {
            Ok(RequiredInfo::new(20, None, Some(1920)))
        }
    } else if pixels >= 921600.0 {
        Ok(RequiredInfo::new(19, None, None))
    } else {
        Ok(RequiredInfo::new(18, None, None))
    }
}

fn generate_output_path(video: &Path) -> Result<PathBuf, EncodeError> {
    let stem = video
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| EncodeError::OutputPath {
            input: video.to_string_lossy().into_owned(),
        })?;

    let new_filename = format!("{stem}-{}.mp4", Local::now().format("%y%m%d%H%M%S"));

    Ok(video.with_file_name(new_filename))
}
