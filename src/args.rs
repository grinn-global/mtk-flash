// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    override_usage = "mtk-flash --da <PATH> [--fip <PATH>] [--img <PATH>] --dev <DEVICE> [--gpio <CHIP>] [--preserve-boot1]",
    about = "A command-line utility for flashing raw images to MediaTek devices.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Args {
    #[clap(long, value_name = "PATH", help = "Download Agent image")]
    pub da: PathBuf,

    #[clap(
        long,
        value_name = "PATH",
        help = "Optional Firmware Image Package image"
    )]
    pub fip: Option<PathBuf>,

    #[clap(long, value_name = "PATH", help = "Optional system image")]
    pub img: Option<PathBuf>,

    #[clap(
        long,
        value_name = "DEVICE",
        help = "Path to the device (e.g. /dev/ttyACM0)"
    )]
    pub dev: String,

    #[clap(
        long,
        value_name = "CHIP",
        help = "Optional path to gpiochip for controlling power, reset and download mode (e.g. /dev/gpiochip0)"
    )]
    pub gpio: Option<PathBuf>,

    #[clap(long, help = "Do not erase mmc0boot1 after flashing FIP.")]
    pub preserve_boot1: bool,
}

impl Args {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
