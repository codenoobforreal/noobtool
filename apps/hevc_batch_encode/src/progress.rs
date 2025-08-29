use anyhow::{Result, bail};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

// frame=338
// fps=104.73
// stream_0_0_q=25.7
// bitrate= 878.8kbits/s
// total_size=1539499
// out_time_us=14014014
// out_time_ms=14014014
// out_time=00:00:14.014014
// dup_frames=0
// drop_frames=0
// speed=4.34x
// progress=end

pub struct ProgressMonitor {
    pb: ProgressBar,
    total_duration_secs: f32,
}

impl ProgressMonitor {
    pub fn new(total_duration_secs: f32) -> Self {
        let pb = ProgressBar::new(100);
        pb.set_draw_target(ProgressDrawTarget::stderr_with_hz(4));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} [{elapsed_precise}] [{bar:40}] {percent}% ({eta})")
                .expect("failed to setup progressbar")
                .progress_chars("#>-"),
        );

        Self {
            pb,
            total_duration_secs,
        }
    }

    #[allow(clippy::lines_filter_map_ok)]
    pub fn process_progress_info(&self, stderr: impl Read) -> Result<(Duration, u64)> {
        if self.total_duration_secs <= 0.0 {
            bail!("Total duration must be greater than zero");
        }

        let mut total_size = 0u64;
        let mut last_progress = 0u8;

        for line in BufReader::new(stderr).lines().filter_map(Result::ok) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "total_size" => total_size = value.parse()?,
                "out_time" => {
                    let current_secs = Self::time_string_to_seconds(value)?;
                    let new_progress =
                        (current_secs / self.total_duration_secs * 100.0).clamp(0.0, 100.0) as u8;

                    if new_progress.abs_diff(last_progress) >= 1 {
                        self.pb.set_position(new_progress.into());
                        last_progress = new_progress;
                    }
                }
                "progress" if value == "end" => {
                    self.pb.finish_and_clear();
                    return Ok((self.pb.elapsed(), total_size));
                }
                _ => {}
            }
        }

        bail!("FFmpeg process ended without completion")
    }

    fn time_string_to_seconds(time_str: &str) -> Result<f32> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            bail!("invalid time string format");
        }

        let hours: f32 = parts[0].parse()?;
        let minutes: f32 = parts[1].parse()?;
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds: f32 = seconds_parts[0].parse()?;

        let microseconds: f32 = match seconds_parts.get(1) {
            Some(micro_str) => micro_str.parse::<f32>()? / 1_000_000.0,
            None => 0.0,
        };

        Ok(hours * 3600.0 + minutes * 60.0 + seconds + microseconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_ffmpeg_output(lines: &[&str]) -> impl std::io::Read {
        let data = lines.join("\n");
        std::io::Cursor::new(data.into_bytes())
    }

    #[test]
    fn test_time_string_conversion() {
        assert_eq!(
            ProgressMonitor::time_string_to_seconds("00:01:30.500000").unwrap(),
            90.5
        );

        assert!(ProgressMonitor::time_string_to_seconds("00:01:30").is_ok());
        assert_eq!(
            ProgressMonitor::time_string_to_seconds("00:01:30").unwrap(),
            90.0
        );

        assert!(ProgressMonitor::time_string_to_seconds("invalid").is_err());
        assert!(ProgressMonitor::time_string_to_seconds("00:01").is_err());

        let result = ProgressMonitor::time_string_to_seconds("abc:01:30");
        assert!(result.is_err());
    }

    #[test]
    fn test_progress_data_parsing() {
        let monitor = ProgressMonitor::new(100.0);
        let stderr = mock_ffmpeg_output(&[
            "total_size=2048000",
            "out_time=00:00:10.000",
            "progress=continue",
            "out_time=00:00:20.000",
            "progress=end",
        ]);

        let (_, total_size) = monitor.process_progress_info(stderr).unwrap();
        assert_eq!(total_size, 2048000);
    }

    #[test]
    fn test_progress_calculation() {
        let monitor = ProgressMonitor::new(200.0);

        // 模拟时间推进：50秒 -> 100秒 -> 150秒
        let stderr = mock_ffmpeg_output(&[
            "out_time=00:00:50.000",
            "out_time=00:01:40.000",
            "out_time=00:02:30.000",
        ]);

        let result = monitor.process_progress_info(stderr);
        assert!(result.is_err());
        assert_eq!(monitor.pb.position(), 75);
    }

    #[test]
    fn test_missing_end_flag() {
        let monitor = ProgressMonitor::new(100.0);
        let stderr = mock_ffmpeg_output(&["total_size=1024000"]);

        let result = monitor.process_progress_info(stderr);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "FFmpeg process ended without completion"
        );
    }
}
