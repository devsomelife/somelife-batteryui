# battery-tui

A 100% terminal battery monitor for Linux (Debian), in the spirit of `glances`/`top`/`ncdu`.

Reads `/sys/class/power_supply` directly (no `upower`, no D-Bus, no root) and renders:

- **Live**: charge gauge, status, power draw (W), voltage, time-to-empty/full, health, AC state, and a recent power-draw sparkline.
- **History**: in-session line charts of charge % and power draw, plus aggregates (avg/min/max draw, energy consumed).
- **Details**: every raw sysfs field as a key/value table.

History is kept in memory for the session only (no persistence, no daemon).

## Build

Requires a Rust toolchain (`cargo`).

```sh
cargo build --release
./target/release/battery-tui
```

Or run directly: `cargo run --release`.

## Usage

```sh
battery-tui                 # auto-detect first battery, 2s interval
battery-tui -i 5            # 5-second sample interval
battery-tui -b BAT1         # select a specific battery
battery-tui --list          # list available batteries and exit
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
