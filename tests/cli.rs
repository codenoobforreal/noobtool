use assert_cmd::Command;
use predicates::prelude::*;

#[test]
#[ignore]
fn error_when_no_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("[INFO] enter 0 paths"));

    Ok(())
}

#[test]
#[ignore]
fn error_when_no_video_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.args(["-i", "."])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "[ERROR] no video found in all your inputs",
        ));

    Ok(())
}

#[test]
#[ignore]
fn print_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Print help (see more with \'--help\')",
        ));

    Ok(())
}

#[test]
#[ignore]
fn print_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());

    Ok(())
}
