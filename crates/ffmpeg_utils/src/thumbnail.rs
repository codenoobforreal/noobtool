use crate::{
    errors::{FfmpegError, ProcessError, ThumbnailError},
    get_metadata,
};
use chrono::Local;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    select,
    time::{Duration, sleep},
};
use tokio_util::sync::CancellationToken;

pub struct Generator {
    input_path: PathBuf,
    output_path: PathBuf,
    cancel_token: CancellationToken,
    spec: Spec,
    timeout: Duration,
}

impl Generator {
    async fn new(
        video_path: PathBuf,
        cancel_token: CancellationToken,
    ) -> Result<Self, ThumbnailError> {
        let output_path = generate_output_path(&video_path)?;
        let required_info = get_required_info(video_path.as_os_str(), cancel_token.clone()).await?;

        // todo: make base_dim configrable
        let thumbnail_spec = Spec::calculate(
            required_info.width,
            required_info.height,
            required_info.duration,
            200.0,
        );

        Ok(Self {
            input_path: video_path,
            output_path,
            spec: thumbnail_spec,
            cancel_token,
            timeout: Duration::from_secs(180),
        })
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
            OsString::from(&self.input_path),
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
            OsString::from(&self.output_path),
        ];

        if spec.skip_frame {
            args.splice(
                2..2,
                [OsString::from("-skip_frame"), OsString::from("nokey")],
            );
        }

        args
    }

    async fn generate(&self) -> Result<(), ThumbnailError> {
        let args = self.build_command_args();

        let mut child = Command::new("ffmpeg")
            .args(args)
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ThumbnailError::Process(ProcessError::Spawn(e.to_string())))?;

        let stderr = child.stderr.take();
        let mut error_str = String::new();

        select! {
            _ = self.cancel_token.cancelled() => {
                match child.kill().await {
                    Ok(()) => Err(ThumbnailError::Process(ProcessError::Canceled)),
                    Err(e) => Err(ThumbnailError::Process(ProcessError::Kill(e.to_string()))),
                }
            }
            _ = sleep(self.timeout) => {
                match child.kill().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ThumbnailError::Ffmpeg(FfmpegError::Timeout(e.to_string()))),
                }
            }
            res = child.wait() => {
                if let Some(mut pipe) = stderr {
                    pipe.read_to_string(&mut error_str).await.ok();
                }
                match res {
                    Ok(status) if status.success() => Ok(()),
                    Ok(_) => Err(ThumbnailError::Ffmpeg(FfmpegError::Inner(error_str))),
                    Err(e) => Err(ThumbnailError::Process(ProcessError::ExitStatus(e.to_string()))),
                }
            }
        }
    }

    // async fn run(&self) -> Result<(), ThumbnailError> {
    //     self.spec
    //         .generate(
    //             self.input_path.as_os_str(),
    //             self.output_path.as_os_str(),
    //             Duration::from_secs(180),
    //             self.cancel_token.clone(),
    //         )
    //         .await
    // }
}

fn generate_output_path(video: &Path) -> Result<PathBuf, ThumbnailError> {
    let stem = video
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or_else(|| {
            ThumbnailError::OutputPath(format!("{}: invalid file name", video.display()))
        })?;

    // note: latest ffmpeg will error when output is jpeg format
    // todo: make format configrable
    let new_filename = format!("{stem}-{}.jpg", Local::now().format("%y%m%d%H%M%S"));

    Ok(video.with_file_name(new_filename))
}

struct RequiredInfo {
    width: f32,
    height: f32,
    duration: f32,
}

impl RequiredInfo {
    fn new(width: f32, height: f32, duration: f32) -> Self {
        RequiredInfo {
            width,
            height,
            duration,
        }
    }
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

    Ok(RequiredInfo::new(
        video_stream.width,
        video_stream.height,
        duration,
    ))
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
        let (width, height) = calc_dimensions(base_dim, video_width, video_height);
        let (rows, cols, start_time, end_time, skip_frame) =
            calc_duration_related_specs(video_duration);

        Self {
            width,
            height,
            rows,
            cols,
            skip_frame,
            start_time,
            end_time,
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

fn calc_dimensions(base_dimension: f32, video_width: f32, video_height: f32) -> (f32, f32) {
    let aspect_ratio = video_width / video_height;

    if aspect_ratio >= 1.0 {
        (base_dimension * aspect_ratio, base_dimension)
    } else {
        (base_dimension, base_dimension * aspect_ratio.recip())
    }
}

fn calc_duration_related_specs(video_duration: f32) -> (usize, usize, f32, f32, bool) {
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

    (
        rows,
        cols,
        start_time,
        video_duration - start_time,
        skip_frame,
    )
}
