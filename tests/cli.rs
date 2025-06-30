use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("debian-genio-flash").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage"));
}

#[test]
fn fails_without_required_args() {
    let mut cmd = Command::cargo_bin("debian-genio-flash").unwrap();
    cmd.assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn fails_with_missing_da() {
    let mut cmd = Command::cargo_bin("debian-genio-flash").unwrap();
    cmd.args(["--dev", "/dev/ttyUSB0"])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn fails_with_missing_dev() {
    let mut cmd = Command::cargo_bin("debian-genio-flash").unwrap();
    cmd.args(["--da", "boot/lk.img"])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}
