// use anyhow::{Result, anyhow, bail};
// use chrono::Local;
// use std::{
//     path::{Path, PathBuf},
//     process::Command,
// };

// pub fn generate_thumbnail(input: &Path, base_dim: u16) -> Result<()> {
//     let config = Config::init(input)?;
//     let metadata = VideoMetadata::retrive(input)?;
//     let spec = Spec::calculate(metadata.width, metadata.height, metadata.duration, base_dim);
//     let generator = Generator::new(config, spec);
//     generator.generate()
// }

// struct Generator {
//     config: Config,
//     spec: Spec,
// }

// impl Generator {
//     fn new(config: Config, spec: Spec) -> Self {
//         Self { config, spec }
//     }

//     fn generate(&self) -> Result<()> {
//         let mut command = Command::new("ffmpeg");

//         command.args(["-hide_banner", "-v", "error"]);

//         let mut vf_string = format!(
//             "fps=1/{:.2},scale={:.2}:{:.2},tile={}x{}",
//             self.spec.interval(),
//             self.spec.width,
//             self.spec.height,
//             self.spec.rows,
//             self.spec.cols
//         );

//         if self.spec.skip_frame {
//             command.args(["-skip_frame", "nokey"]);
//             vf_string.insert_str(0, "select='eq(pict_type,I)',");
//         }

//         command.args([
//             "-ss",
//             &format!("{:.2}", self.spec.start_time),
//             "-to",
//             &format!("{:.2}", self.spec.end_time),
//             "-i",
//         ]);

//         command.arg(&self.config.input);

//         command.args([
//             "-map",
//             "0:v",
//             "-vf",
//             &vf_string,
//             "-fps_mode",
//             "vfr",
//             "-frames:v",
//             "1",
//             "-update",
//             "1",
//             "-q:v",
//             "2",
//             "-y",
//         ]);

//         command.arg(&self.config.output.0);

//         let output = command.output()?;

//         if !output.status.success() {
//             let error_msg = if output.stderr.is_empty() {
//                 format!("FFprobe exited with status {}", output.status)
//             } else {
//                 String::from_utf8_lossy(&output.stderr).into_owned()
//             };
//             bail!("FFprobe error: {}", error_msg);
//         }

//         Ok(())
//     }
// }

// struct OutputPath(PathBuf);

// impl OutputPath {
//     fn from_path(path: &Path) -> Result<Self> {
//         let stem = path
//             .file_stem()
//             .ok_or_else(|| anyhow!("failed to get stem portion of {}", path.display()))?;

//         // note: latest ffmpeg will error when output is jpeg format
//         // todo: find available pic formats
//         let new_filename = format!(
//             "{}-{}.jpg",
//             stem.to_string_lossy(),
//             Local::now().format("%y%m%d%H%M%S")
//         );

//         Ok(Self(path.with_file_name(new_filename)))
//     }
// }

// struct Config {
//     input: PathBuf,
//     output: OutputPath,
// }

// impl Config {
//     fn init(input: &Path) -> Result<Self> {
//         let output = OutputPath::from_path(input)?;
//         Ok(Config {
//             input: input.to_path_buf(),
//             output,
//         })
//     }
// }

// struct VideoMetadata {
//     width: u16,
//     height: u16,
//     duration: f32,
// }

// // ​-show_entries stream=nb_read_frames​
// impl VideoMetadata {
//     fn retrive(video: &Path) -> Result<Self> {
//         // ffprobe -v error -select_streams v:0 -show_entries stream=width,height -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 input.mp4
//         let output = Command::new("ffprobe")
//             .args([
//                 "-v",
//                 "error",
//                 "-select_streams",
//                 "v:0",
//                 "-show_entries",
//                 "stream=width,height",
//                 "-show_entries",
//                 "format=duration",
//                 "-of",
//                 "default=noprint_wrappers=1",
//             ])
//             .arg(video)
//             .output()?;

//         if !output.status.success() {
//             let error_msg = if output.stderr.is_empty() {
//                 format!("FFprobe exited with status {}", output.status)
//             } else {
//                 String::from_utf8_lossy(&output.stderr).into_owned()
//             };
//             bail!("FFprobe error: {}", error_msg);
//         }

//         let out_str = String::from_utf8(output.stdout)?;

//         let mut width = None;
//         let mut height = None;
//         let mut duration = None;

//         for line in out_str.lines() {
//             match line {
//                 s if s.starts_with("width=") => {
//                     width = Some(line.trim_start_matches("width=").parse::<u16>()?)
//                 }
//                 s if s.starts_with("height=") => {
//                     height = Some(line.trim_start_matches("height=").parse::<u16>()?)
//                 }

//                 s if s.starts_with("duration=") => {
//                     duration = Some(line.trim_start_matches("duration=").parse::<f32>()?)
//                 }

//                 _ => (),
//             };
//         }

//         let width = width.ok_or_else(|| anyhow!("missing width in metadata"))?;
//         let height = height.ok_or_else(|| anyhow!("missing height in metadata"))?;
//         let duration = duration.ok_or_else(|| anyhow!("missing duration in metadata"))?;

//         Ok(VideoMetadata {
//             width,
//             height,
//             duration,
//         })
//     }
// }

// #[derive(Debug)]
// struct Spec {
//     width: f32,
//     height: f32,
//     rows: u8,
//     cols: u8,
//     skip_frame: bool,
//     start_time: f32,
//     end_time: f32,
// }

// impl Spec {
//     fn calculate(video_width: u16, video_height: u16, video_duration: f32, base_dim: u16) -> Self {
//         let aspect_ratio = video_width as f32 / video_height as f32;

//         let (width, height) = if aspect_ratio >= 1.0 {
//             (base_dim as f32 * aspect_ratio, base_dim as f32)
//         } else {
//             (base_dim as f32, base_dim as f32 * aspect_ratio.recip())
//         };

//         let (rows, cols, cut_percent, skip_frame) = match video_duration {
//             // 4 grid needs 60s for each (GOP) so it is 4*60=240
//             d if d <= 240.0 => (2, 2, 0.05, false),
//             d if d <= 600.0 => (3, 2, 0.05, true),
//             d if d <= 1800.0 => (3, 3, 0.04, true),
//             d if d <= 3600.0 => (4, 3, 0.03, true),
//             d if d <= 7200.0 => (4, 4, 0.02, true),
//             d if d <= 14400.0 => (5, 4, 0.01, true),
//             _ => (5, 5, 0.005, true),
//         };

//         let start_time = video_duration * cut_percent;

//         Self {
//             width,
//             height,
//             rows,
//             cols,
//             skip_frame,
//             start_time,
//             end_time: video_duration - start_time,
//         }
//     }

//     fn grid_count(&self) -> u8 {
//         self.rows * self.cols
//     }

//     fn duration(&self) -> f32 {
//         self.end_time - self.start_time
//     }

//     fn interval(&self) -> f32 {
//         self.duration() / self.grid_count() as f32
//     }
// }
