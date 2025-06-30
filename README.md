# Grinn Flash Tool

**Grinn Flash** is a command-line utility for flashing firmware to Grinn Genio devices via USB and UART. It supports MediaTek boot ROM (BROM) communication, DA upload, and fastboot flashing with image splitting.

## Usage

```sh
grinn-flash --da <PATH> --fip <PATH> --dev <DEVICE> [--img <PATH>]
```

### Arguments

* `--da <PATH>`: Path to the Download Agent image.
* `--fip <PATH>`: Path to the Firmware Image Package (FIP) image.
* `--img <PATH>` *(optional)*: Path to the system image.
* `--dev <DEVICE>`: Serial device path.

### Example

```sh
sudo grinn-flash \
  --da boot/lk.img \
  --fip boot/fip.img \
  --img mediatek-genio-emmc.sparse.img \
  --dev /dev/ttyACM0
```

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE.md) file for details.
