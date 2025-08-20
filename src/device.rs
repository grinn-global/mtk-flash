// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use anyhow::{Context, Result};
use fastboot_protocol::nusb::{NusbFastBoot, devices};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use mediatek_brom::{Brom, io::BromExecuteAsync};
use std::{path::Path, thread::sleep, time::Duration};
use tokio::fs;
use tokio_serial::SerialPortBuilderExt;

const BAUD_RATE: u32 = 115200;
const HANDSHAKE_ADDRESS: u32 = 0x201000;

pub struct DeviceControl {
    reset: LineHandle,
    download: LineHandle,
    power: LineHandle,
}

impl DeviceControl {
    pub fn new(
        chip_path: &Path,
        reset_line: u32,
        download_line: u32,
        power_line: u32,
    ) -> Result<Self> {
        let chip_str = chip_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in chip path"))?;

        let mut chip = Chip::new(chip_str)
            .with_context(|| format!("Failed to open GPIO chip at {chip_str}"))?;

        let reset = chip
            .get_line(reset_line)?
            .request(LineRequestFlags::OUTPUT, 0, "reset")?;

        let download =
            chip.get_line(download_line)?
                .request(LineRequestFlags::OUTPUT, 0, "download")?;

        let power = chip
            .get_line(power_line)?
            .request(LineRequestFlags::OUTPUT, 0, "power")?;

        Ok(DeviceControl {
            reset,
            download,
            power,
        })
    }

    pub fn reset(&self) -> Result<()> {
        self.reset.set_value(1)?;
        sleep(Duration::from_millis(100));
        self.reset.set_value(0)?;
        Ok(())
    }

    pub fn download_mode(&self) -> Result<()> {
        self.download.set_value(1)?;
        sleep(Duration::from_millis(100));
        self.reset()?;
        sleep(Duration::from_millis(100));
        self.download.set_value(0)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn power(&self) -> Result<()> {
        self.power.set_value(1)?;
        sleep(Duration::from_secs(1));
        self.power.set_value(0)?;
        Ok(())
    }
}

pub async fn initialize_brom(da_path: &Path, dev_path: &str) -> Result<()> {
    println!("Waiting for target device...");
    while !Path::new(dev_path).exists() {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let mut serial = tokio_serial::new(dev_path, BAUD_RATE).open_native_async()?;
    let brom = serial.execute(Brom::handshake(HANDSHAKE_ADDRESS)).await?;
    let hwcode = serial.execute(brom.hwcode()).await?;
    println!(
        "\nHandshake successful: SoC 0x{:04x}, version {}.",
        hwcode.code, hwcode.version
    );

    let data = fs::read(da_path).await?;
    println!("Uploading DA to {HANDSHAKE_ADDRESS:#x}...");
    serial.execute(brom.send_da(&data)).await?;
    println!("Executing DA...");
    serial.execute(brom.jump_da64()).await?;

    Ok(())
}

pub async fn wait_for_fastboot() -> Result<NusbFastBoot> {
    println!("\nWaiting for fastboot device...");
    let device = loop {
        if let Some(d) = devices()?.next() {
            break d;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    Ok(NusbFastBoot::from_info(&device)?)
}
