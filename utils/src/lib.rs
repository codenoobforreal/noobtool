mod constants;
mod file;
mod format;
mod parse;
mod path;

pub use file::scan_videos_from_paths;
pub use format::{format_duration, format_file_size};
pub use parse::parse_fraction;
pub use path::{find_videos_within_folder, is_root_path, is_video_path, resolve_to_absolute};

use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

pub fn get_command_args(command: &Command) -> OsString {
    command.get_args().collect::<Vec<_>>().join(OsStr::new(" "))
}
