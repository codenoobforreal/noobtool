// fn generate_thumbnails(videos: &[PathBuf], base_dim: u16) -> bool {
//     let mut errors = vec![];

//     let pb = setup_progress_bar(
//         videos
//             .len()
//             .try_into()
//             .expect("[ERROR] Video count too large"),
//     );

//     pb.set_position(0);
//     pb.set_message(format!("{}", videos.first().unwrap().display()));

//     videos.iter().enumerate().for_each(|(index, video)| {
//         match generator::generate_thumbnail(video, base_dim) {
//             Ok(_) if index + 1 < videos.len() => {
//                 pb.inc(1);
//                 pb.set_message(format!("{}", videos[index + 1].display()));
//             }
//             Ok(_) => pb.inc(1),
//             Err(e) => errors.push(e),
//         }
//     });

//     let success_count = pb.position();
//     let duration = pb.elapsed();

//     pb.finish_and_clear();

//     errors.iter().for_each(|e| log::error!("{e}"));

//     log::info!(
//         "generated {success_count} thumbnails in {} with {} failures",
//         format_duration(duration),
//         errors.len(),
//     );

//     !errors.is_empty()
// }

// fn setup_progress_bar(len: u64) -> ProgressBar {
//     let pb = ProgressBar::new(len);
//     pb.set_draw_target(ProgressDrawTarget::stderr_with_hz(4));
//     pb.set_style(
//         ProgressStyle::default_bar()
//             .template(
//                 "{spinner} [{elapsed_precise}] [{bar:40}] {percent}% ({eta}) {pos}/{len} | {msg}",
//             )
//             .unwrap()
//             .progress_chars("#>-"),
//     );
//     pb
// }
