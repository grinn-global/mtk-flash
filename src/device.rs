use anyhow::Result;
use fastboot_protocol::nusb::{NusbFastBoot, devices};
use mediatek_brom::{Brom, io::BromExecuteAsync};
use std::path::Path;
use tokio::fs;
use tokio_serial::SerialPortBuilderExt;

const BAUD_RATE: u32 = 115200;
const HANDSHAKE_ADDRESS: u32 = 0x201000;
const TARGET_FASTBOOT_ID: &str = "0123456789ABCDEF";

pub async fn initialize_brom(da_path: &Path, dev_path: &str) -> Result<()> {
    println!("Waiting for target device...\n");
    while !Path::new(dev_path).exists() {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let mut serial = tokio_serial::new(dev_path, BAUD_RATE).open_native_async()?;
    let brom = serial.execute(Brom::handshake(HANDSHAKE_ADDRESS)).await?;
    let hwcode = serial.execute(brom.hwcode()).await?;
    println!(
        "Handshake successful: SoC 0x{:04x}, version {}",
        hwcode.code, hwcode.version
    );

    let data = fs::read(da_path).await?;
    println!("Uploading DA to {HANDSHAKE_ADDRESS:#x}...");
    serial.execute(brom.send_da(&data)).await?;
    println!("Executing DA...\n");
    serial.execute(brom.jump_da64()).await?;

    Ok(())
}

pub async fn wait_for_fastboot() -> Result<NusbFastBoot> {
    println!("Waiting for fastboot device...\n");
    let device = loop {
        if let Some(d) = devices()?.find(|d| {
            d.serial_number()
                .map(|s| s == TARGET_FASTBOOT_ID)
                .unwrap_or(false)
        }) {
            break d;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };

    Ok(NusbFastBoot::from_info(&device)?)
}
