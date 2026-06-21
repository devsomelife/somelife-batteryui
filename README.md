# Some-BatteryUI

[![CI](https://github.com/devsomelife/somelife-batteryui/actions/workflows/ci.yml/badge.svg)](https://github.com/devsomelife/somelife-batteryui/actions/workflows/ci.yml)
[![Release](https://github.com/devsomelife/somelife-batteryui/actions/workflows/release.yml/badge.svg)](https://github.com/devsomelife/somelife-batteryui/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/devsomelife/somelife-batteryui)](https://github.com/devsomelife/somelife-batteryui/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A 100% terminal battery monitor for Linux (Debian), in the spirit of `glances`/`top`/`ncdu`.

Reads `/sys/class/power_supply` directly (no `upower`, no D-Bus, no root) and renders:

- **Live**: charge gauge, status, power draw (W), voltage, time-to-empty/full, health, AC state, and a recent power-draw sparkline.
- **History**: in-session line charts of charge % and power draw, plus aggregates (avg/min/max draw, energy consumed).
- **Details**: every raw sysfs field as a key/value table.

History is kept in memory for the session only (no persistence, no daemon).

## Install

### Download a prebuilt binary (no Rust needed)

Grab the static `x86_64` Linux binary from the [Releases](https://github.com/devsomelife/somelife-batteryui/releases) page. It has no dependencies and runs on any Linux:

```sh
curl -L -o some-batteryui https://github.com/devsomelife/somelife-batteryui/releases/latest/download/some-batteryui-x86_64-linux
chmod +x some-batteryui
./some-batteryui
```

### Debian / Ubuntu (`.deb`)

Download the `.deb` from Releases and install it:

```sh
sudo apt install ./some-batteryui_*_amd64.deb
some-batteryui
```

### Build from source

Requires a Rust toolchain (`cargo`).

```sh
cargo build --release
./target/release/some-batteryui
```

For a fully static, portable binary:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Usage

```sh
some-batteryui                 # auto-detect first battery, 2s interval
some-batteryui -i 5            # 5-second sample interval
some-batteryui -b BAT1         # select a specific battery
some-batteryui --list          # list available batteries and exit
```

## Keys

| Key         | Action                    |
|-------------|---------------------------|
| `q` / `Esc` | quit                      |
| `Tab` / `1`–`3` | switch view           |
| `p`         | pause / resume sampling   |
| `r`         | reset history             |
| `+` / `-`   | adjust sample interval    |

## Notes

- Supports both the `energy_*`/`power_now` (µWh/µW) and `charge_*`/`current_now`
  (µAh/µA) sysfs families; power and health are derived when not exposed directly.
- On systems without a battery (desktops), it shows a clear message and still runs.
