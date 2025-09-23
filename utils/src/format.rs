use std::time::Duration;

pub fn format_file_size(bytes: impl Into<u64>) -> String {
    let bytes_u64: u64 = bytes.into();
    const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    if bytes_u64 == 0 {
        return "0B".to_string();
    }
    let mut size = bytes_u64 as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    let formatted = format!("{:.2}", size);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{}{}", trimmed, UNITS[unit_index])
}

pub fn format_duration(d: Duration) -> String {
    const SECONDS_PER_DAY: f64 = 86400.0;
    const SECONDS_PER_HOUR: f64 = 3600.0;
    const SECONDS_PER_MINUTE: f64 = 60.0;

    let total_seconds = d.as_secs_f64();

    if total_seconds >= SECONDS_PER_DAY {
        let days = total_seconds / SECONDS_PER_DAY;
        format!("{:.1}d", days)
    } else if total_seconds >= SECONDS_PER_HOUR {
        let hours = total_seconds / SECONDS_PER_HOUR;
        format!("{:.1}h", hours)
    } else if total_seconds >= SECONDS_PER_MINUTE {
        let minutes = total_seconds / SECONDS_PER_MINUTE;
        format!("{:.1}m", minutes)
    } else {
        format!("{:.1}s", total_seconds)
    }
}
