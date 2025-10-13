use crate::{Config, EncoderError, error::EncodeResult};
use ffmpeg_command_builder::FfmpegCommandBuilder;
use ffmpeg_progress_monitor::ProgressMonitor;
use std::{
    cmp::{Ordering, min},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use video_metadata::{Metadata, Orientation, Resolution};

#[derive(Debug, PartialEq, Clone)]
pub struct Encoder<'a> {
    input: &'a Path,
    output: &'a Path,
    // preset: Preset,
    crf: u8,
    fps: Option<u8>,
    scaled_width: Option<u16>,
    scaled_height: Option<u16>,
}

impl<'a> Encoder<'a> {
    pub fn new(config: &'a Config, metadata: &Metadata) -> EncodeResult<Self> {
        let fps = if metadata.fps() > config.fps().into() {
            Some(config.fps())
        } else {
            None
        };

        let (crf, scaled_width, scaled_height) = Self::compute_scaling_params(config, metadata)?;

        Ok(Self {
            input: config.input,
            output: config.output,
            // preset: config.preset(),
            crf,
            fps,
            scaled_width,
            scaled_height,
        })
    }

    /// 计算编码缩放参数（CRF和可选的缩放宽高）
    ///
    /// # 策略
    /// - 分辨率下降时（元数据分辨率≥配置）：根据视频朝向调整宽高，并使用配置的CRF
    /// - 分辨率上升时（元数据分辨率<配置）：不缩放宽高，使用元数据的CRF
    fn compute_scaling_params(
        config: &Config,
        metadata: &Metadata,
    ) -> EncodeResult<(u8, Option<u16>, Option<u16>)> {
        match metadata.pixels().cmp(&config.resolution().pixels()) {
            Ordering::Greater | Ordering::Equal => {
                // 分辨率下降逻辑
                let crf = resolution_to_crf(config.resolution());
                let orientation = metadata.resolution()?.get_orientation();
                let (scaled_width, scaled_height) = match orientation {
                    Orientation::Landscape => {
                        let width = config.resolution().get_primary_dimension();
                        (Some(width), None)
                    }
                    Orientation::Portrait => {
                        let height = config.resolution().get_primary_dimension();
                        (None, Some(height))
                    }
                };
                Ok((crf, scaled_width, scaled_height))
            }
            Ordering::Less => {
                // 分辨率上升逻辑
                let crf = resolution_to_crf(metadata.resolution()?);
                Ok((crf, None, None))
            }
        }
    }

    /// 构建视频编码所需要的 `Command`
    ///
    /// # ffmpeg命令举例
    /// ffmpeg -hide_banner -v error -progress pipe:2 -i input.mp4 -c:v libsvtav1 -preset 4 -crf 32 -g 240 -svtav1-params tune=0:film-grain=4 -vf scale=1280:-2,fps=24 -c:a copy output.mp4
    ///
    /// # 参考文档
    /// https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/master/Docs/Ffmpeg.md
    /// https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/master/Docs/Parameters.md
    pub(crate) fn build_ffmpeg_command(&self) -> EncodeResult<Command> {
        let mut builder = FfmpegCommandBuilder::new()
            .global_opt("-hide_banner -v error -progress pipe:2")
            .input(self.input.to_string_lossy())
            .output_opt("-c:v libsvtav1 -preset 4 -crf")
            .output_opt(self.crf.to_string())
            .output_opt(format!(
                "-g {} -svtav1-params tune=0:film-grain=4",
                self.gop()
            ));

        if let Some(vf_str) = self.video_filter() {
            builder = builder.output_opt(format!("-vf {}", vf_str));
        }

        let command = builder.output(self.output.to_string_lossy()).build();

        Ok(command)
    }

    fn gop(&self) -> u16 {
        match self.fps {
            Some(fps) => min((fps as u16) * 10, 300),
            // 这个值是 cli 的 fps 参数的默认值的 10 倍
            None => 240,
        }
    }

    fn video_filter(&self) -> Option<String> {
        let scale_str = match (self.scaled_width, self.scaled_height) {
            (Some(w), None) => Some(format!("scale={}:-2", w)),
            (None, Some(h)) => Some(format!("scale=-2:{}", h)),
            _ => None,
        };

        let fps_str = self.fps.map(|f| format!("fps={}", f));

        match (scale_str, fps_str) {
            (Some(scale), Some(fps)) => Some(format!("{},{}", scale, fps)),
            (None, Some(fps)) => Some(fps),
            (Some(scale), None) => Some(scale),
            _ => None,
        }
    }

    pub fn encode(&self, monitor: ProgressMonitor) -> EncodeResult<(Duration, u64)> {
        let mut command = self.build_ffmpeg_command()?;

        let mut child = command.stderr(Stdio::piped()).spawn()?;

        let stderr = child.stderr.take().ok_or(EncoderError::TakeStd)?;

        let result = monitor.process_progress_info(stderr)?;

        let status = child.wait()?;
        if !status.success() {
            return Err(EncoderError::FfmpegExit(format!("{}", status)));
        }

        Ok(result)
    }

    // pub fn preset(&self) -> Preset {
    //     self.preset
    // }

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

impl<'a> Default for Encoder<'a> {
    fn default() -> Self {
        Self {
            input: Path::new("input.mp4"),
            output: Path::new("output.mp4"),
            crf: Default::default(),
            fps: Default::default(),
            scaled_width: Default::default(),
            scaled_height: Default::default(),
        }
    }
}

/// https://handbrake.fr/docs/en/1.9.0/workflow/adjust-quality.html
fn resolution_to_crf(resolution: Resolution) -> u8 {
    match resolution.pixels() {
        p if p >= Resolution::Hd.pixels() => 25,
        _ => 22,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use utils::get_command_args;

    #[test]
    fn source_downscale_to_config() -> EncodeResult<()> {
        // 源视频横屏，配置横屏
        let metadata = Metadata::new(1_920, 1_080, 30.0, 0.0, 0);
        let config = Config {
            resolution: Resolution::Hd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"));
        assert!(args.contains("-vf scale=1280:-2,fps=24"));

        // 源视频横屏，配置竖屏
        let config = Config {
            resolution: Resolution::Vhd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"));
        assert!(args.contains("-vf scale=1280:-2,fps=24"));

        // 源视频竖屏，配置竖屏
        let metadata = Metadata::new(1_080, 1_920, 30.0, 0.0, 0);
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"));
        assert!(args.contains("-vf scale=-2:1280,fps=24"), "{}", args);

        // 源视频竖屏，配置竖屏
        let config = Config {
            resolution: Resolution::Hd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"));
        assert!(args.contains("-vf scale=-2:1280,fps=24"));

        Ok(())
    }

    #[test]
    fn source_use_default() -> EncodeResult<()> {
        // 横屏
        let metadata = Metadata::new(1_920, 1_080, 24.0, 0.0, 0);
        let config = Config {
            fps: 24,
            resolution: Resolution::Qhd,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"), "{}", args);
        assert!(!args.contains("-vf"));

        // 竖屏
        let config = Config {
            resolution: Resolution::Vqhd,
            fps: 24,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 240"));
        assert!(!args.contains("-vf"));

        // 没有缩放但有fps限制
        let config = Config {
            resolution: Resolution::Vqhd,
            fps: 20,
            ..Config::default()
        };
        let encoder = Encoder::new(&config, &metadata)?;
        let command = encoder.build_ffmpeg_command()?;
        let args = get_command_args(&command).to_string_lossy().to_string();
        assert!(args.contains("-crf 25 -g 200"));
        assert!(args.contains("-vf fps=20"));

        Ok(())
    }
}
