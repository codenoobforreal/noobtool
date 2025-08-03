use once_cell::sync::Lazy;
use std::collections::HashSet;

pub const VIDEO_EXTS: [&str; 12] = [
    "ts", "3gp", "ogg", "avi", "flv", "m4v", "mkv", "mov", "mp4", "wmv", "rmvb", "webm",
];

pub static VIDEO_EXTS_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| VIDEO_EXTS.into_iter().collect());
