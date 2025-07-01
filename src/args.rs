// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    override_usage = "debian-genio-flash --da <PATH> [--fip <PATH>] [--img <PATH>] --dev <DEVICE>",
    about = "A tool for flashing Grinn Genio devices.",
    version = "0.1.0"
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
}

impl Args {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
