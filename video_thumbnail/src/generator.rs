use crate::{Grid, ThumbnailError, error::ThumbnailResult};
use chrono::Local;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub struct Generator<'a> {
    input: &'a Path,
    duration: f32,
    grid: Grid,
    base_dimesion: u16,
    ratio: f32,
}

impl<'a> Generator<'a> {
    pub fn new(input: &'a Path, duration: f32, grid: Grid, base_dimesion: u16, ratio: f32) -> Self {
        Self {
            input,
            duration,
            grid,
            base_dimesion,
            ratio,
        }
    }

    pub(crate) fn build_ffmpeg_args(&self) -> ThumbnailResult<Vec<String>> {
        let (width, height) = self.calc_dimension();

        let mut args: Vec<String> = Vec::new();

        args.extend(
            [
                "-hide_banner",
                "-v",
                "error",
                "-progress",
                "pipe:2",
                "-skip_frame",
                "nokey",
                "-i",
            ]
            .iter()
            .map(|&s| s.to_string()),
        );

        args.push(self.input.to_string_lossy().into_owned());

        args.extend(["-map", "0:v", "-vf"].iter().map(|&s| s.to_string()));

        let (row, col) = match self.grid {
            Grid { row: 0, col: 0 } => self.get_default_settings(),
            g => (g.row, g.col),
        };

        args.push(format!(
            "select='eq(pict_type,I)',fps=1/{},scale={}:{},tile={}x{}",
            self.interval(),
            width,
            height,
            row,
            col
        ));

        args.extend(
            [
                "-fps_mode",
                "vfr",
                "-frames:v",
                "1",
                "-update",
                "1",
                "-q:v",
                "2",
                "-y",
            ]
            .iter()
            .map(|&s| s.to_string()),
        );

        args.push(self.output()?.to_string_lossy().into_owned());

        Ok(args)
    }

    pub fn generate(&self) -> ThumbnailResult<()> {
        let mut command = Command::new("ffmpeg");

        command.args(self.build_ffmpeg_args()?);

        let mut child = command.stderr(Stdio::piped()).spawn()?;

        // let stderr = child.stderr.take().ok_or(ThumbnailError::TakeStd)?;

        let status = child.wait()?;
        if !status.success() {
            return Err(ThumbnailError::FfmpegExit(format!("{}", status)));
        }

        Ok(())
    }

    fn interval(&self) -> u32 {
        // self.duration / self.grid_count() as f32

        let grid_count = match self.grid {
            Grid { row: 0, col: 0 } => {
                let (r, c) = self.get_default_settings();
                r as u16 * c as u16
            }
            _ => self.grid.count(),
        };

        (self.duration / ((grid_count + 1) as f32)) as u32
    }

    fn calc_dimension(&self) -> (u16, u16) {
        if self.ratio >= 1.0 {
            (
                (self.base_dimesion as f32 * self.ratio) as u16,
                self.base_dimesion,
            )
        } else {
            (
                self.base_dimesion,
                (self.base_dimesion as f32 * self.ratio.recip()) as u16,
            )
        }
    }

    fn get_default_settings(&self) -> (u8, u8) {
        match self.duration {
            // 4 grid needs 60s for each (GOP) so it is 4*60=240
            d if d <= 240.0 => (2, 2),
            d if d <= 600.0 => (3, 2),
            d if d <= 1800.0 => (3, 3),
            d if d <= 3600.0 => (4, 3),
            d if d <= 7200.0 => (4, 4),
            d if d <= 14400.0 => (5, 4),
            _ => (5, 5),
        }
    }

    fn output(&self) -> ThumbnailResult<PathBuf> {
        let stem = self
            .input
            .file_stem()
            .ok_or_else(|| ThumbnailError::FileStem(self.input.to_string_lossy().to_string()))?;

        let new_filename = format!(
            "{}-{}.jpg",
            stem.display(),
            Local::now().format("%y%m%d%H%M%S")
        );

        Ok(self.input.with_file_name(new_filename))
    }
}
