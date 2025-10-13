use crate::{Grid, ThumbnailError, error::ThumbnailResult};
use ffmpeg_command_builder::FfmpegCommandBuilder;
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub struct Generator<'a> {
    input: &'a Path,
    output: &'a Path,
    duration: f32,
    grid: Grid,
    base_dimesion: u16,
    ratio: f32,
}

impl<'a> Generator<'a> {
    pub fn new(
        input: &'a Path,
        output: &'a Path,
        duration: f32,
        grid: Grid,
        base_dimesion: u16,
        ratio: f32,
    ) -> Self {
        Self {
            input,
            output,
            duration,
            grid,
            base_dimesion,
            ratio,
        }
    }

    pub(crate) fn build_ffmpeg_command(&self) -> ThumbnailResult<Command> {
        let (width, height) = self.calc_dimension();
        let (row, col) = match self.grid {
            Grid { row: 0, col: 0 } => self.get_default_grid_config(),
            g => (g.row, g.col),
        };

        Ok(FfmpegCommandBuilder::new()
            .global_opt("-hide_banner -v error -skip_frame nokey -y")
            .input(self.input.to_string_lossy())
            .output_opt("-map 0:v")
            .output_opt(format!(
                "-vf select='eq(pict_type,I)',fps=1/{},scale={}:{},tile={}x{}",
                self.interval(),
                width,
                height,
                row,
                col
            ))
            .output_opt("-fps_mode vfr -frames:v 1 -update 1 -q:v 2")
            .output(self.output.to_string_lossy())
            .build())
    }

    pub fn generate(&self) -> ThumbnailResult<()> {
        let mut child = self
            .build_ffmpeg_command()?
            .stderr(Stdio::piped())
            .spawn()?;

        // let mut output = String::new();
        // let _ = child.stderr.take().unwrap().read_to_string(&mut output);
        // println!("{}", output);

        let status = child.wait()?;
        if !status.success() {
            return Err(ThumbnailError::FfmpegExit(format!("{}", status)));
        }

        Ok(())
    }

    fn interval(&self) -> u32 {
        let grid_count = match self.grid {
            Grid { row: 0, col: 0 } => {
                let (r, c) = self.get_default_grid_config();
                r as u16 * c as u16
            }
            _ => self.grid.count(),
        };

        (self.duration / ((grid_count + 1) as f32)) as u32
    }

    fn calc_dimension(&self) -> (u16, u16) {
        match self.ratio >= 1.0 {
            true => (
                (self.base_dimesion as f32 * self.ratio) as u16,
                self.base_dimesion,
            ),
            false => (
                self.base_dimesion,
                (self.base_dimesion as f32 * self.ratio.recip()) as u16,
            ),
        }
    }

    fn get_default_grid_config(&self) -> (u8, u8) {
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
}
