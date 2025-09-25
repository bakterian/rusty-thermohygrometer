# Rusty Thermohygrometer

## Introduction
This is a embedded rust source code repository.</br>
The compiled program is flashed and executed on a ESP32C3 RISC-V board.

## TODO
The ESP32-C3 Super Mini board manages multiple meteorological sensors.</br>
The device is connected to the Internet via a local WiFi connection.</br>
Thanks to which all of the measurments are periodically uploaded to a remote MQTT Broker.

## Configuration
Before building please provide of the necessary conifguration data in `.cargo/config.toml`</br>
The configuration parameters are things like:
- WiFi-ssid
- WiFi-password
- Mqtt-broker ip-address
- Mqtt-broker port
- MQTT temperature-data topic
- MQTT humidity-data topic


## Build environment preparation
The following steps assume a Linux operating system with small adjustments the same steps can be performed i.e. on Windows</br>
We'll now use [Rustup](https://rustup.rs) to install both [Rust](https://github.com/rust-lang/rust) and [Cargo](https://github.com/rust-lang/cargo) (Rust's package manager):

```bash
curl --proto '=https' --tlsv1.2 --fail --show-error --silent https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup toolchain install nightly --component rust-src
```

Moreover, we need a few additional Cargo modules:

- `espflash` to flash the device (see [espflash](https://github.com/esp-rs/espflash))
- `ldproxy` to forward linker arguments (see [ldproxy](https://github.com/esp-rs/embuild/tree/f2cbbf9795676af52d2ffb53f102d70cac25116a/ldproxy))
- `cargo-generate` to generate projects according to a template (see [cargo-generate](https://github.com/cargo-generate/cargo-generate))

```bash
cargo install espflash ldproxy cargo-generate
```

Getting access to the USB peripheral

When connecting the ESP32-C3 via USB you might see "Permission Denied" error to avoid those add this udev rule:

```bash
echo "SUBSYSTEMS==\"usb\", ATTRS{idVendor}==\"303a\", ATTRS{idProduct}==\"1001\", MODE=\"0660\", GROUP=\"plugdev\"" | sudo tee /etc/udev/rules.d/99-esp-rust-board.rules > /dev/null
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Building
When the build environment is ready and all of the congifuration-data was filled the rust embedded application can be build.<br>
In the terminal access the checkoud sources and type:
```bash
cargo build
```

## Firmware updates
Connect the ESP32-C3 via USB, in the terminal access the checkoud sources and do:
```bash
cargo run
```