<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>
-->

# MTK Flash

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/grinn-global/mtk-flash/main.yml) ![License](https://img.shields.io/github/license/grinn-global/mtk-flash)

**MTK Flash** is a command-line utility for flashing raw images to MediaTek devices, supporting MediaTek Boot ROM communication, Download Agent upload, and Fastboot flashing with image splitting.

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
   ./target/release/mtk-flash --help
   ```

Alternatively, a precompiled binary is available on the [Releases](https://github.com/grinn-global/mtk-flash/releases) page.

## Usage

1. Add your user to the `dialout` group:

    ```sh
    sudo usermod -aG dialout $USER
    ```

2. If you want to use the automatic boot into download mode functionality, you also need to set up udev rules for the GPIO chip:

    ```sh
    sudo cp data/90-mtk-flash.rules /etc/udev/rules.d/
    sudo udevadm control --reload && sudo udevadm trigger
    ```

3. Log out and log back in to apply the group membership. Alternatively, you can run `newgrp dialout` in the terminal to apply the changes immediately.

4. Use the tool with the correct serial device path:

    ```sh
    mtk-flash \
        --da lk.bin \
        --fip fip.img \
        --img emmc_sparse.img \
        --dev /dev/ttyACM0
    ```

### Arguments

```sh
mtk-flash --da <PATH> [--fip <PATH>] [--img <PATH>] --dev <DEVICE> [--gpio <CHIP>]
```

- `--da <PATH>`: Path to the Download Agent image.
- `--fip <PATH>` *(optional)*: Path to the Firmware Image Package (FIP) image.
- `--img <PATH>` *(optional)*: Path to the system image.
- `--dev <DEVICE>`: Serial device path.
- `--gpio <CHIP>` *(optional)*: Path to the GPIO chip device for controlling power, reset and download mode.

## Troubleshooting

If you encounter issues, ensure that:
- The device is connected and recognized by the system.
- You have the necessary permissions to access the serial device.
- The specified paths to the images are correct.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE.md) file for details.
