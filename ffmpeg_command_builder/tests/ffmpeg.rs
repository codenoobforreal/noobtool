use ffmpeg_command_builder::FfmpegCommandBuilder;
use std::ffi::OsString;
use utils::get_command_args;

#[test]
fn build_command() {
    let ffmpeg_command = FfmpegCommandBuilder::new()
        .global_opt("-hide_banner -v error -progress pipe:2")
        .input("input.mp4")
        .input_opt("-ss 00:00:10")
        .output_opt("-c:v libx265 -c:a copy")
        .output("output.mp4")
        .build();

    let args = get_command_args(&ffmpeg_command);

    assert_eq!(
        args,
        OsString::from(
            "-hide_banner -v error -progress pipe:2 -ss 00:00:10 -i input.mp4 -c:v libx265 -c:a copy output.mp4"
        ),
        "{:#?}",
        args
    );
}
