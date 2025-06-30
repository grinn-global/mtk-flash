use std::path::{Path, PathBuf};

use android_sparse_image::{
    CHUNK_HEADER_BYTES_LEN, ChunkHeader, FileHeader, FileHeaderBytes, ParseError,
    split::{split_image, split_raw},
};
use anyhow::{Context, Result, bail};
use clap::Parser;
use fastboot_protocol::{
    nusb::{NusbFastBoot, devices},
    protocol::parse_u32_hex,
};
use indicatif::{ProgressBar, ProgressStyle};
use mediatek_brom::{Brom, io::BromExecuteAsync};
use nix::unistd::Uid;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
};
use tokio_serial::SerialPortBuilderExt;

#[derive(Parser)]
#[clap(
    override_usage = "grinn-flash --da <PATH> --fip <PATH> --dev <DEVICE> [--img <PATH>]",
    about = "A tool for flashing Grinn Genio devices.",
    version = "0.1.0"
)]
struct Args {
    #[clap(long, value_name = "PATH", help = "Download Agent image")]
    da: PathBuf,

    #[clap(long, value_name = "PATH", help = "Firmware Image Package image")]
    fip: PathBuf,

    #[clap(long, value_name = "PATH", help = "Optional system image")]
    img: Option<PathBuf>,

    #[clap(
        long,
        value_name = "DEVICE",
        help = "Path to the device (e.g. /dev/ttyUSB0)"
    )]
    dev: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    if !Uid::effective().is_root() {
        eprintln!("Error: this tool must be run as root.");
        std::process::exit(1);
    }

    let args = Args::try_parse().unwrap_or_else(|e| {
        let mut msg = e.to_string();
        msg = msg.replace(
            "The following required arguments were not provided:",
            "Missing required arguments:",
        );
        eprintln!("{}", msg);
        std::process::exit(1);
    });

    println!("Waiting for target device...");
    while !Path::new(&args.dev).exists() {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    {
        let baud_rate = 115200;
        let address = 0x201000;

        let mut serial = tokio_serial::new(&args.dev, baud_rate).open_native_async()?;

        let brom = serial.execute(Brom::handshake(address)).await?;
        let hwcode = serial.execute(brom.hwcode()).await?;
        println!(
            "Handshake successful: (SoC 0x{:04x}, version {})",
            hwcode.code, hwcode.version
        );

        let data = tokio::fs::read(&args.da).await?;
        println!("Uploading DA to {:#x}...", 0x201000);
        serial.execute(brom.send_da(&data)).await?;
        println!("Executing DA...");
        serial.execute(brom.jump_da64()).await?;
    }

    println!("\nWaiting for fastboot device...");
    let device = loop {
        if let Some(d) = devices()?.next() {
            break d;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };
    let mut fb = NusbFastBoot::from_info(&device)?;

    println!("\nFlashing FIP to mmc0boot0...");
    flash(&mut fb, "mmc0boot0", &args.fip).await?;

    println!("Erasing mmc0boot1...");
    fb.erase("mmc0boot1").await?;

    if let Some(img) = &args.img {
        println!("\nFlashing IMG to mmc0...");
        fb.erase("mmc0").await?;
        flash(&mut fb, "mmc0", img).await?;
    } else {
        println!("No system image provided, skipping mmc0 erase and flash.");
    }

    println!("All operations completed successfully.");
    Ok(())
}

async fn flash_raw(fb: &mut NusbFastBoot, target: &str, mut file: File, size: u32) -> Result<()> {
    let mut sender = fb.download(size).await?;
    let mut left = size as usize;
    let pb = ProgressBar::new(left as u64).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.green/black}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );

    while left > 0 {
        let buf = sender.get_mut_data(left).await?;
        file.read_exact(buf).await?;
        left -= buf.len();
        pb.inc(buf.len() as u64);
    }

    pb.finish_with_message("Upload complete");
    sender.finish().await?;
    println!("\nFlashing data...");
    fb.flash(target).await?;
    Ok(())
}

async fn flash(fb: &mut NusbFastBoot, target: &str, path: &Path) -> Result<()> {
    let max_dl = fb.get_var("max-download-size").await?;
    let max_download = parse_u32_hex(&max_dl).context("Failed to parse max download size")?;

    let mut file = File::open(path).await?;
    let mut header_bytes = FileHeaderBytes::default();
    file.read_exact(&mut header_bytes).await?;

    let splits = match FileHeader::from_bytes(&header_bytes) {
        Ok(header) => {
            let mut chunks = Vec::with_capacity(header.chunks as usize);
            for _ in 0..header.chunks {
                let mut chunk_bytes = [0u8; CHUNK_HEADER_BYTES_LEN];
                file.read_exact(&mut chunk_bytes).await?;
                let chunk = ChunkHeader::from_bytes(&chunk_bytes)?;
                file.seek(SeekFrom::Current(chunk.data_size() as i64))
                    .await?;
                chunks.push(chunk);
            }
            split_image(&header, &chunks, max_download)?
        }
        Err(ParseError::UnknownMagic) => {
            file.seek(SeekFrom::Start(0)).await?;
            let size = file.seek(SeekFrom::End(0)).await?;
            if size < max_download as u64 {
                file.seek(SeekFrom::Start(0)).await?;
                return flash_raw(fb, target, file, size as u32).await;
            }
            split_raw(size as usize, max_download)?
        }
        Err(e) => bail!("Failed to parse image: {e}"),
    };

    println!("Uploading in {} parts", splits.len());
    for (i, split) in splits.iter().enumerate() {
        println!("Uploading part {}", i);
        let mut sender = fb.download(split.sparse_size() as u32).await?;
        sender.extend_from_slice(&split.header.to_bytes()).await?;

        let total_bytes: usize = split
            .chunks
            .iter()
            .map(|c| c.header.to_bytes().len() + c.size)
            .sum();
        let pb = ProgressBar::new(total_bytes as u64).with_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] [{bar:40.green/black}] {bytes}/{total_bytes} ({eta})",
                )
                .unwrap()
                .progress_chars("=>-"),
        );

        for chunk in &split.chunks {
            let hdr = chunk.header.to_bytes();
            sender.extend_from_slice(&hdr).await?;
            pb.inc(hdr.len() as u64);

            file.seek(SeekFrom::Start(chunk.offset as u64)).await?;
            let mut remaining = chunk.size;
            while remaining > 0 {
                let buf = sender.get_mut_data(remaining).await?;
                let n = file.read_exact(buf).await?;
                remaining -= n;
                pb.inc(n as u64);
            }
        }

        pb.finish_with_message("Part uploaded");
        sender.finish().await?;
        fb.flash(target).await?;
    }

    Ok(())
}
