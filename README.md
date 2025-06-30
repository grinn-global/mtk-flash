# Debian Genio Flash

**Debian Genio Flash** is a command-line utility for flashing Debian-based firmware to Grinn Genio devices via USB and UART. It supports MediaTek boot ROM (BROM) communication, DA upload, and fastboot flashing with image splitting.

## Usage

1. **Add user to `dialout` group**:

    ```sh
    sudo usermod -aG dialout <username>
    ```

2. **Log out and log back in** for group membership to apply.

3. **Use the tool** with correct device path. Example:

    ```sh
    sudo debian-grinn-flash \
      --da boot/lk.img \
      --fip boot/fip.img \
      --img emmc_sparse.img \
      --dev /dev/ttyUSB0
    ```

### Arguments

```sh
debian-grinn-flash --da <PATH> [--fip <PATH>] [--img <PATH>] --dev <DEVICE>
```

* `--da <PATH>`: Path to the Download Agent image.
* `--fip <PATH>` *(optional)*: Path to the Firmware Image Package (FIP) image.
* `--img <PATH>` *(optional)*: Path to the system image.
* `--dev <DEVICE>`: Serial device path.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE.md) file for details.
