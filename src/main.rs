// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

mod args;
mod device;
mod flash;
mod interrupt;

use anyhow::Result;
use args::Args;
use interrupt::{InterruptState, setup_interrupt_handler};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let interrupt_state = Arc::new(Mutex::new(InterruptState::new()));
    setup_interrupt_handler(interrupt_state.clone());

    device::initialize_brom(&args.da, &args.dev).await?;
    let mut fb = device::wait_for_fastboot().await?;

    if let Some(fip) = &args.fip {
        println!("\nFlashing FIP to mmc0boot0...");
        flash::flash(&mut fb, "mmc0boot0", fip, interrupt_state.clone()).await?;

        println!("\nErasing mmc0boot1...");
        fb.erase("mmc0boot1").await?;
    } else {
        println!("No FIP image provided, skipping mmc0boot0 flash.");
    }

    if let Some(img) = &args.img {
        println!("\nErasing mmc0...");
        fb.erase("mmc0").await?;
        println!("\nFlashing IMG to mmc0...");
        flash::flash(&mut fb, "mmc0", img, interrupt_state.clone()).await?;
    } else {
        println!("No system image provided, skipping mmc0 flash.");
    }

    println!("\nAll operations completed successfully.");
    Ok(())
}
