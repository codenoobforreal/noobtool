use crate::{
    constants::{CRF_DEFAULT, CRF_FHD, CRF_HD, DEFAULT_FPS, FHD_PIXELS, FHD_WIDTH, HD_PIXELS},
    progress::ProgressMonitor,
};
use anyhow::{Result, anyhow, bail};
use chrono::Local;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use utils::{format_duration, format_file_size, parse_fraction};

pub fn process_encode(input: &Path) -> Result<()> {
    let config = Config::init(input)?;
    let metadata = VideoMetadata::retrive(input)?;
    let spec = Spec::init(metadata.width, metadata.height, metadata.fps);
    let encoder = Encoder::new(config, spec);
    let stat = encoder.encode(ProgressMonitor::new(metadata.duration))?;

    log::info!(
        "{:?} encoded in {} and shrunk(maybe) to {} ({:.2}% of original)",
        input.file_name().unwrap_or(input.as_os_str()),
        format_duration(stat.0),
        format_file_size(stat.1),
        (stat.1 as f64 / metadata.size as f64) * 100.0
    );

    Ok(())
}

struct OutputPath(PathBuf);

impl OutputPath {
    fn from_path(path: &Path) -> Result<Self> {
        let stem = path
            .file_stem()
            .ok_or_else(|| anyhow!("failed to get stem portion of {}", path.display()))?;

        // note: latest ffmpeg will error when output is jpeg format
        // todo: find available pic formats
        let new_filename = format!(
            "{}-{}.mp4",
            stem.to_string_lossy(),
            Local::now().format("%y%m%d%H%M%S")
        );

        Ok(Self(path.with_file_name(new_filename)))
    }
}

struct Config {
    input: PathBuf,
    output: OutputPath,
}

impl Config {
    fn init(input: &Path) -> Result<Self> {
        let output = OutputPath::from_path(input)?;
        Ok(Config {
            input: input.to_path_buf(),
            output,
        })
    }
}

struct Encoder {
    config: Config,
    spec: Spec,
}

impl Encoder {
    fn new(config: Config, spec: Spec) -> Self {
        Self { config, spec }
    }

    fn scale_filter(&self) -> Option<[String; 2]> {
        match (self.spec.scaled_width, self.spec.scaled_height) {
            (Some(width), None) => Some(["-vf".to_string(), format!("scale={}:-2", width)]),
            (None, Some(height)) => Some(["-vf".to_string(), format!("scale=-2:{}", height)]),
            _ => None,
        }
    }

    fn encode(&self, monitor: ProgressMonitor) -> Result<(Duration, u64)> {
        // ffmpeg -hide_banner -v error -progress pipe:2 -i input.mp4 -c:v libx265 -x265-params log-level=error:output-depth=10:ctu=32:merange=32:crf=20 -pix_fmt yuv420p10le -filter:v fps=24 -f mp4 -c:a copy output.mp4
        let mut command = Command::new("ffmpeg");

        command
            .args(["-hide_banner", "-v", "error", "-progress", "pipe:2", "-i"])
            .arg(&self.config.input)
            .args(["-c:v", "libx265", "-x265-params"])
            .arg(format!(
                "log-level=error:output-depth=10:ctu=32:merange=32:crf={}",
                self.spec.crf
            ))
            .args(["-preset", "slow", "-pix_fmt", "yuv420p10le"]);

        if let Some(filter) = self.scale_filter() {
            command.args(filter);
        }

        if let Some(fps) = self.spec.fps {
            command.arg("-filter:v");
            command.arg(format!("fps={}", fps));
        }

        let mut child = command
            .args(["-f", "mp4", "-c:a", "copy"])
            .arg(&self.config.output.0)
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("failed to get stderr"))?;

        let result = monitor.process_progress_info(stderr);

        let status = child.wait()?;
        if !status.success() {
            bail!("FFmpeg exited with status {}", status);
        }

        result
    }
}

struct VideoMetadata {
    width: u16,
    height: u16,
    fps: f32,      // 平均帧率
    duration: f32, // 秒
    size: u64,     // 字节
}

impl VideoMetadata {
    fn retrive(video: &Path) -> Result<Self> {
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
                format!("FFprobe exited with status {}", output.status)
            } else {
                String::from_utf8_lossy(&output.stderr).into_owned()
            };
            bail!("FFprobe error: {}", error_msg);
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

        let width = width.ok_or_else(|| anyhow!("missing width in metadata"))?;
        let height = height.ok_or_else(|| anyhow!("missing height in metadata"))?;
        let fps = fps.ok_or_else(|| anyhow!("missing fps in metadata"))?;
        let duration = duration.ok_or_else(|| anyhow!("missing duration in metadata"))?;
        let size = size.ok_or_else(|| anyhow!("missing size in metadata"))?;

        Ok(VideoMetadata {
            width,
            height,
            fps,
            duration,
            size,
        })
    }
}

struct Spec {
    crf: u8,
    fps: Option<u8>,
    scaled_width: Option<u16>,
    scaled_height: Option<u16>,
}

impl Spec {
    fn init(width: u16, height: u16, fps: f32) -> Self {
        let fps = if fps > DEFAULT_FPS.into() {
            Some(DEFAULT_FPS)
        } else {
            None
        };

        let (crf, scaled_width, scaled_height) = {
            let pixels = width as u32 * height as u32;
            if pixels >= FHD_PIXELS {
                (
                    CRF_FHD,
                    if width >= height {
                        Some(FHD_WIDTH)
                    } else {
                        None
                    },
                    if width < height {
                        Some(FHD_WIDTH)
                    } else {
                        None
                    },
                )
            } else if pixels >= HD_PIXELS {
                (CRF_HD, None, None)
            } else {
                (CRF_DEFAULT, None, None)
            }
        };

        Self {
            crf,
            fps,
            scaled_width,
            scaled_height,
        }
    }
}
