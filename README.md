<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>
-->

# Debian Genio Flash

![CI](https://github.com/grinn-global/debian-genio-flash/actions/workflows/main.yml/badge.svg)

**Debian Genio Flash** is a command-line utility for flashing Debian-based firmware to Grinn Genio devices via USB and UART. It supports MediaTek boot ROM (BROM) communication, Download Agent upload, and Fastboot flashing with image splitting.

## Building

1. Install dependencies:

   ```sh
   sudo apt install -y rustc cargo build-essential
   ```

2. Build the project:

   ```sh
   cargo build --release
   ```

   The compiled binary will be located in the `target/release` directory:

   ```
   ./target/release/debian-genio-flash --help
   ```

Alternatively, a precompiled binary is available on the [Releases](https://github.com/grinn-global/debian-genio-flash/releases) page.

## Usage

1. Add your user to the `dialout` group:

    ```sh
    sudo usermod -aG dialout <username>
    ```

2. Log out and log back in to apply the group membership. Alternatively, you can run `newgrp dialout` in the terminal to apply the changes immediately.

3. Use the tool with the correct serial device path:

    ```sh
    sudo debian-genio-flash \
        --da boot/lk.img \
        --fip boot/fip.img \
        --img emmc_sparse.img \
        --dev /dev/ttyUSB0
    ```

### Arguments

```sh
debian-genio-flash --da <PATH> [--fip <PATH>] [--img <PATH>] --dev <DEVICE>
```

- `--da <PATH>`: Path to the Download Agent image.
- `--fip <PATH>` *(optional)*: Path to the Firmware Image Package (FIP) image.
- `--img <PATH>` *(optional)*: Path to the system image.
- `--dev <DEVICE>`: Serial device path.

## Troubleshooting

If you encounter issues, ensure that:
- The device is connected and recognized by the system.
- You have the necessary permissions to access the serial device.
- The specified paths to the images are correct.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE.md) file for details.
