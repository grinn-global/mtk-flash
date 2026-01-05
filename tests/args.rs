// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use assert_cmd::Command;
use clap::Parser;
use mtk_flash::args::Args;
use predicates::str::contains;

#[test]
fn parses_all_args() {
    let args = Args::parse_from([
        "test",
        "--da",
        "boot/lk.img",
        "--fip",
        "boot/fip.img",
        "--img",
        "system.img",
        "--dev",
        "/dev/ttyUSB0",
    ]);
    assert_eq!(args.da.to_str().unwrap(), "boot/lk.img");
    assert_eq!(args.fip.unwrap().to_str().unwrap(), "boot/fip.img");
    assert_eq!(args.img.unwrap().to_str().unwrap(), "system.img");
    assert_eq!(args.dev, "/dev/ttyUSB0");
}

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("mtk-flash").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage"));
}

#[test]
fn fails_without_required_args() {
    let mut cmd = Command::cargo_bin("mtk-flash").unwrap();
    cmd.assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn fails_with_missing_da() {
    let mut cmd = Command::cargo_bin("mtk-flash").unwrap();
    cmd.args(["--dev", "/dev/ttyUSB0"])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn fails_with_missing_dev() {
    let mut cmd = Command::cargo_bin("mtk-flash").unwrap();
    cmd.args(["--da", "boot/lk.img"])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn parses_no_erase_boot1_flag() {
    let args = Args::parse_from([
        "test",
        "--da",
        "boot/lk.img",
        "--dev",
        "/dev/ttyUSB0",
        "--preserve-boot1",
    ]);
    assert!(args.preserve_boot1);

    let args = Args::parse_from(["test", "--da", "boot/lk.img", "--dev", "/dev/ttyUSB0"]);
    assert!(!args.preserve_boot1);
}
