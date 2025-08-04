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
use tokio::{
    io::AsyncReadExt,
    process::Command,
    select,
    time::{Duration, sleep},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum ThumbnailError {
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

pub async fn process_thumbnail_generate(
    cancel_token: CancellationToken,
    timeout: Duration,
    input: PathBuf,
    base_dim: f32,
) -> Result<(), ThumbnailError> {
    let token_clone = cancel_token.clone();
    select! {
        _ = token_clone.cancelled() => Err(ProcessError::Canceled)?,
        res = async {
          let config = Config::init(input, timeout)?;
          let info = get_required_info(config.input.as_os_str(), cancel_token.clone()).await?;
          let spec = Spec::calculate(info.width, info.height, info.duration, base_dim);
          let generator = Generator::new(cancel_token, config, spec);
          generator.run().await
        } => res,
    }
}

struct Generator {
    cancel_token: CancellationToken,
    config: Config,
    spec: Spec,
}

impl Generator {
    fn new(cancel_token: CancellationToken, config: Config, spec: Spec) -> Self {
        Self {
            cancel_token,
            config,
            spec,
        }
    }

    async fn run(&self) -> Result<(), ThumbnailError> {
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
            _ = sleep(self.config.timeout) => {
                match child.kill().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(FfmpegError::Timeout(e.to_string()))?,
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

    fn build_command_args(&self) -> Vec<OsString> {
        let spec = &self.spec;
        let mut args = vec![
            OsString::from("-v"),
            OsString::from("fatal"),
            OsString::from("-ss"),
            OsString::from(format!("{:.2}", spec.start_time)),
            OsString::from("-to"),
            OsString::from(format!("{:.2}", spec.end_time)),
            OsString::from("-i"),
            OsString::from(&self.config.input),
            OsString::from("-map"),
            OsString::from("0:v"),
            OsString::from("-vf"),
            OsString::from(format!(
                "fps=1/{:.2},scale={:.2}:{:.2},tile={}x{}",
                spec.interval(),
                spec.width,
                spec.height,
                spec.rows,
                spec.cols
            )),
            OsString::from("-fps_mode"),
            OsString::from("vfr"),
            OsString::from("-frames:v"),
            OsString::from("1"),
            OsString::from("-update"),
            OsString::from("1"),
            OsString::from("-q:v"),
            OsString::from("2"),
            OsString::from("-y"),
            OsString::from(&self.config.output),
        ];

        if spec.skip_frame {
            args.splice(
                2..2,
                [OsString::from("-skip_frame"), OsString::from("nokey")],
            );
        }

        args
    }
}

fn generate_output_path(video: &Path) -> Result<PathBuf, ThumbnailError> {
    let stem = video
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| ThumbnailError::OutputPath {
            input: video.to_string_lossy().into_owned(),
        })?;

    // note: latest ffmpeg will error when output is jpeg format
    // todo: find available pic formats
    let new_filename = format!("{stem}-{}.jpg", Local::now().format("%y%m%d%H%M%S"));

    Ok(video.with_file_name(new_filename))
}

struct Config {
    input: PathBuf,
    output: PathBuf,
    timeout: Duration,
}

impl Config {
    fn init(input: PathBuf, timeout: Duration) -> Result<Self, ThumbnailError> {
        let output = generate_output_path(&input)?;
        Ok(Config {
            input,
            output,
            timeout,
        })
    }
}

struct RequiredInfo {
    width: f32,
    height: f32,
    duration: f32,
}

async fn get_required_info(
    video: &OsStr,
    cancel_token: CancellationToken,
) -> Result<RequiredInfo, ThumbnailError> {
    let display_path = video.display();
    let metadata = get_metadata(video, cancel_token).await?;
    let duration = metadata.format.duration.0;
    let video_stream = metadata.first_stream().ok_or_else(|| {
        ThumbnailError::MissingField(format!("{display_path} failed to get stream info"))
    })?;

    Ok(RequiredInfo {
        width: video_stream.width,
        height: video_stream.height,
        duration,
    })
}

#[derive(Debug)]
struct Spec {
    width: f32,
    height: f32,
    rows: usize,
    cols: usize,
    skip_frame: bool,
    start_time: f32,
    end_time: f32,
}

impl Spec {
    fn calculate(video_width: f32, video_height: f32, video_duration: f32, base_dim: f32) -> Self {
        let aspect_ratio = video_width / video_height;

        let (width, height) = if aspect_ratio >= 1.0 {
            (base_dim * aspect_ratio, base_dim)
        } else {
            (base_dim, base_dim * aspect_ratio.recip())
        };

        let (rows, cols, cut_percent, skip_frame) = match video_duration {
            d if d <= 240.0 => (2, 2, 0.05, false),
            d if d <= 600.0 => (3, 2, 0.05, true),
            d if d <= 1800.0 => (3, 3, 0.04, true),
            d if d <= 3600.0 => (4, 3, 0.03, true),
            d if d <= 7200.0 => (4, 4, 0.02, true),
            d if d <= 14400.0 => (5, 4, 0.01, true),
            _ => (5, 5, 0.005, true),
        };

        let start_time = video_duration * cut_percent;

        Self {
            width,
            height,
            rows,
            cols,
            skip_frame,
            start_time,
            end_time: video_duration - start_time,
        }
    }

    fn grid_count(&self) -> usize {
        self.rows * self.cols
    }

    fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }

    fn interval(&self) -> f32 {
        self.duration() / self.grid_count() as f32
    }
}
