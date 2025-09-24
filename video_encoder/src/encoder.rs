use crate::{Config, EncoderError, error::EncodeResult, preset::Preset};
use chrono::Local;
use ffmpeg_progress_monitor::ProgressMonitor;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use video_metadata::{Metadata, Resolution};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Encoder {
    input: PathBuf,
    // output: PathBuf,
    preset: Preset,
    crf: u8,
    fps: Option<u8>,
    scaled_width: Option<u16>,
    scaled_height: Option<u16>,
}

impl Encoder {
    pub fn new(config: &Config, metadata: &Metadata) -> EncodeResult<Self> {
        let fps = if metadata.fps() > config.fps().into() {
            Some(config.fps())
        } else {
            None
        };

        let (crf, scaled_width, scaled_height) = {
            if metadata.pixels() >= config.resolution().pixels() {
                (
                    resolution_to_crf(config.resolution()),
                    if config.resolution().width() >= config.resolution().height() {
                        Some(config.resolution().width())
                    } else {
                        None
                    },
                    if config.resolution().width() < config.resolution().height() {
                        Some(config.resolution().height())
                    } else {
                        None
                    },
                )
            } else {
                (resolution_to_crf(metadata.resolution()?), None, None)
            }
        };

        Ok(Self {
            input: config.input().to_path_buf(),
            preset: config.preset(),
            crf,
            fps,
            scaled_width,
            scaled_height,
        })
    }

    // ffmpeg -hide_banner -v error -progress pipe:2 -i input.mp4 -c:v libx265 -x265-params log-level=error:output-depth=10:crf=20 -pix_fmt yuv420p10le -preset medium -vf scale=1280:-2,fps=24 -f mp4 -c:a copy output.mp4
    pub(crate) fn build_ffmpeg_args(&self) -> EncodeResult<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        args.extend(
            ["-hide_banner", "-v", "error", "-progress", "pipe:2", "-i"]
                .iter()
                .map(|&s| s.to_string()),
        );

        args.push(self.input.to_string_lossy().into_owned());
        args.extend(
            ["-c:v", "libx265", "-x265-params"]
                .iter()
                .map(|&s| s.to_string()),
        );

        args.push(format!("log-level=error:output-depth=10:crf={}", self.crf));

        args.push("-preset".to_string());

        args.push(format!("{}", self.preset));

        args.extend(["-pix_fmt", "yuv420p10le"].iter().map(|&s| s.to_string()));

        match (self.scaled_width, self.scaled_height, self.fps) {
            (Some(width), None, Some(fps)) => {
                args.push("-vf".to_string());
                args.push(format!("scale={}:-2,fps={}", width, fps));
            }
            (Some(width), None, None) => {
                args.push("-vf".to_string());
                args.push(format!("scale={}:-2", width));
            }
            (None, Some(height), Some(fps)) => {
                args.push("-vf".to_string());
                args.push(format!("scale=-2:{},fps={}", height, fps));
            }
            (None, Some(height), None) => {
                args.push("-vf".to_string());
                args.push(format!("scale=-2:{}", height));
            }
            _ => (),
        }

        args.extend(["-f", "mp4", "-c:a", "copy"].iter().map(|&s| s.to_string()));

        args.push(self.output()?.to_string_lossy().into_owned());

        Ok(args)
    }

    pub fn encode(&self, monitor: ProgressMonitor) -> EncodeResult<(Duration, u64)> {
        let mut command = Command::new("ffmpeg");

        command.args(self.build_ffmpeg_args()?);

        let mut child = command.stderr(Stdio::piped()).spawn()?;

        let stderr = child.stderr.take().ok_or(EncoderError::TakeStd)?;

        let result = monitor.process_progress_info(stderr)?;

        let status = child.wait()?;
        if !status.success() {
            return Err(EncoderError::FfmpegExit(format!("{}", status)));
        }

        Ok(result)
    }

    fn output(&self) -> EncodeResult<PathBuf> {
        let stem = self
            .input
            .file_stem()
            .ok_or_else(|| EncoderError::FileStem(self.input.to_string_lossy().to_string()))?;

        // todo: 支持更多格式
        let new_filename = format!(
            "{}-{}.mp4",
            stem.display(),
            Local::now().format("%y%m%d%H%M%S")
        );

        Ok(self.input.with_file_name(new_filename))
    }

    pub fn input(&self) -> PathBuf {
        self.input.clone()
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }

    pub fn crf(&self) -> u8 {
        self.crf
    }

    pub fn fps(&self) -> Option<u8> {
        self.fps
    }

    pub fn scaled_width(&self) -> Option<u16> {
        self.scaled_width
    }

    pub fn scaled_height(&self) -> Option<u16> {
        self.scaled_height
    }
}

/// https://handbrake.fr/docs/en/1.9.0/workflow/adjust-quality.html
fn resolution_to_crf(resolution: Resolution) -> u8 {
    match resolution.pixels() {
        p if p >= Resolution::Qhd.pixels() => 22,
        p if p >= Resolution::Fhd.pixels() => 20,
        p if p >= Resolution::Hd.pixels() => 19,
        _ => 18,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn use_config() -> EncodeResult<()> {
        let metadata = Metadata::new(1_920, 1_080, 30.0, 0.0, 0);
        let config = Config {
            input: "/path/to/video".into(),
            resolution: Resolution::Hd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        assert_eq!(encoder.fps, Some(config.fps));
        assert_eq!(encoder.crf, 19);
        assert_eq!(encoder.scaled_width, Some(config.resolution.width()));
        assert_eq!(encoder.scaled_height, None);
        let args = encoder.build_ffmpeg_args()?.join(" ");
        assert!(args.contains(&"crf=19".to_string()));
        assert!(args.contains(&format!(
            "-vf scale={}:-2,fps={}",
            config.resolution.width(),
            config.fps
        )));

        // 竖屏
        let config = Config {
            input: "/path/to/video".into(),
            resolution: Resolution::Vhd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        assert_eq!(encoder.scaled_width, None);
        assert_eq!(encoder.scaled_height, Some(config.resolution().height()));
        let args = encoder.build_ffmpeg_args()?.join(" ");
        assert!(args.contains(&format!("-vf scale=-2:{}", config.resolution.height())));
        Ok(())
    }

    #[test]
    fn use_input() -> EncodeResult<()> {
        let metadata = Metadata::new(1_920, 1_080, 24.0, 0.0, 0);
        let config = Config {
            input: "/path/to/video".into(),
            fps: 30,
            resolution: Resolution::Qhd,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        assert_eq!(encoder.fps, None);
        assert_eq!(encoder.crf, 20);
        assert_eq!(encoder.scaled_width, None);
        assert_eq!(encoder.scaled_height, None);
        let args = encoder.build_ffmpeg_args()?.join(" ");
        assert!(args.contains(&"crf=20".to_string()));
        assert!(!args.contains(&"-vf".to_string()));

        // 竖屏
        let config = Config {
            input: "/path/to/video".into(),
            resolution: Resolution::Vqhd,
            fps: 30,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        assert_eq!(encoder.scaled_width, None);
        assert_eq!(encoder.scaled_height, None);
        let args = encoder.build_ffmpeg_args()?.join(" ");
        assert!(!args.contains(&"-vf".to_string()));

        Ok(())
    }
}
