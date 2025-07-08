// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use crate::interrupt::InterruptState;
use android_sparse_image::{
    CHUNK_HEADER_BYTES_LEN, ChunkHeader, FileHeader, FileHeaderBytes, ParseError,
    split::{split_image, split_raw},
};
use anyhow::{Context, Result, bail};
use fastboot_protocol::{nusb::NusbFastBoot, protocol::parse_u32_hex};
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::Path, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
    sync::Mutex,
    time::{Duration, Instant, sleep},
};

pub async fn flash(
    fb: &mut NusbFastBoot,
    target: &str,
    path: &Path,
    interrupt_state: Arc<Mutex<InterruptState>>,
) -> Result<()> {
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
                return flash_raw(fb, target, file, size as u32, interrupt_state).await;
            }
            split_raw(size as usize, max_download)?
        }
        Err(e) => bail!("Failed to parse image: {e}"),
    };

    println!("Uploading in {} parts", splits.len());

    let mut last_duration: Option<Duration> = None;

    for split in &splits {
        {
            let state = interrupt_state.lock().await;
            if state.confirmed_abort {
                bail!("Flashing aborted by user.");
            }
        }

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
            {
                let state = interrupt_state.lock().await;
                if state.confirmed_abort {
                    bail!("Flashing aborted by user.");
                }
            }

            let hdr = chunk.header.to_bytes();
            sender.extend_from_slice(&hdr).await?;
            pb.inc(hdr.len() as u64);

            file.seek(SeekFrom::Start(chunk.offset as u64)).await?;
            let mut remaining = chunk.size;
            while remaining > 0 {
                {
                    let state = interrupt_state.lock().await;
                    if state.confirmed_abort {
                        bail!("Flashing aborted by user.");
                    }
                }

                let buf = sender.get_mut_data(remaining).await?;
                let n = file.read_exact(buf).await?;
                remaining -= n;
                pb.inc(n as u64);
            }
        }

        pb.finish_with_message("Part uploaded");
        sender.finish().await?;

        let estimated = last_duration.unwrap_or(Duration::from_secs(12)); // A guess for the first iteration
        let total_ticks = estimated.as_millis() as u64;
        let flash_pb = ProgressBar::new(total_ticks).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/black}] Writing {wide_msg}")
                .unwrap()
                .progress_chars("=>-"),
        );

        let pb_clone = flash_pb.clone();
        let tick = tokio::spawn(async move {
            let start = Instant::now();
            while start.elapsed() < estimated {
                pb_clone.inc(100);
                sleep(Duration::from_millis(100)).await;
            }
        });

        let start = Instant::now();
        fb.flash(target).await?;
        let duration = start.elapsed();
        last_duration = Some(duration);
        flash_pb.finish_with_message("[OK]");
        tick.abort();
    }

    println!(); // Ensure newline after progress bar

    Ok(())
}

async fn flash_raw(
    fb: &mut NusbFastBoot,
    target: &str,
    mut file: File,
    size: u32,
    interrupt_state: Arc<Mutex<InterruptState>>,
) -> Result<()> {
    let mut sender = fb.download(size).await?;
    let mut left = size as usize;
    let pb = ProgressBar::new(left as u64).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.green/black}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );

    while left > 0 {
        {
            let state = interrupt_state.lock().await;
            if state.confirmed_abort {
                bail!("Flashing aborted by user.");
            }
        }

        let buf = sender.get_mut_data(left).await?;
        file.read_exact(buf).await?;
        left -= buf.len();
        pb.inc(buf.len() as u64);
    }

    pb.finish_with_message("Upload complete");
    sender.finish().await?;
    fb.flash(target).await?;

    println!(); // Ensure newline after progress bar

    Ok(())
}
