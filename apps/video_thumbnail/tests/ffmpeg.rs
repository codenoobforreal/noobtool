mod common;

use assert_cmd::Command;
use common::{CleanupGuard, TEST_VIDEOS, fixtures_path, skip_if_in_ci};
use serial_test::serial;

#[test]
#[serial(ffmpeg)]
fn relative_path() -> Result<(), Box<dyn std::error::Error>> {
    skip_if_in_ci()?;

    let _guard = CleanupGuard;
    let pkg_name = env!("CARGO_PKG_NAME");
    let mut cmd = Command::cargo_bin(pkg_name)?;

    cmd.args([
        "-i",
        &format!("../../apps/{}/fixtures/{}", pkg_name, TEST_VIDEOS[0]),
    ])
    .assert()
    .success();

    Ok(())
}

#[test]
#[serial(ffmpeg)]
fn absolute_path() -> Result<(), Box<dyn std::error::Error>> {
    skip_if_in_ci()?;

    let _guard = CleanupGuard;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.args(["-i", fixtures_path().join(TEST_VIDEOS[0]).to_str().unwrap()])
        .assert()
        .success();

    Ok(())
}
