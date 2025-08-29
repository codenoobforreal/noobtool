use env_logger::{Env, WriteStyle};
use hevc_batch_encode::run;
use std::{io::Write, process};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .write_style(WriteStyle::Never)
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    match run() {
        Ok(has_error) if has_error => process::exit(1),
        Ok(_) => process::exit(0),
        Err(e) => {
            log::error!("{}", e);
            process::exit(1);
        }
    }
}
