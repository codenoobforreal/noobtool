use video_metadata::Resolution;
// use crate::preset::Preset;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Config<'a> {
    /// 输入视频路径
    pub(crate) input: &'a Path,
    // /// 输出视频路径
    pub(crate) output: &'a Path,
    /// 分辨率限制，若输入视频分辨率高于该分辨率则限制到该分辨率，低于该分辨率则使用源视频分辨率
    pub(crate) resolution: Resolution,
    /// 编码器预设
    // pub(crate) preset: Preset,
    /// 帧率
    pub(crate) fps: u8,
}

impl<'a> Config<'a> {
    pub fn init(
        input: &'a Path,
        output: &'a Path,
        resolution: Resolution,
        // preset: Preset,
        fps: u8,
    ) -> Self {
        Config {
            input,
            output,
            resolution,
            // preset,
            fps,
        }
    }

    pub fn input(&self) -> &Path {
        self.input
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    // pub fn preset(&self) -> Preset {
    //     self.preset
    // }

    pub fn fps(&self) -> u8 {
        self.fps
    }
}

#[allow(clippy::derivable_impls)]
impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            input: Path::new("input.mp4"),
            output: Path::new("output.mp4"),
            resolution: Resolution::default(),
            // preset: Preset::default(),
            fps: Default::default(),
        }
    }
}
