# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`battery-tui` — a terminal battery monitor for Linux (Debian), in the spirit of `glances`/`top`. Reads `/sys/class/power_supply` directly. No `upower`, no D-Bus, no root, no persistence. History lives in memory for the session only.

## Commands

Requires a Rust toolchain newer than the system's. It was installed via rustup; source it first in a fresh shell:

```sh
. "$HOME/.cargo/env"
```

- Build: `cargo build` (debug) / `cargo build --release`
- Run: `cargo run --release` or `./target/release/battery-tui`
- The TUI needs a real terminal (alternate screen). Don't run it through a pipe — it won't render and key input won't reach it.

Before pushing, match what CI enforces (it fails on either): `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. Run `cargo fmt` to fix formatting.

Non-interactive paths useful for smoke-testing without a TTY:
- `./target/release/battery-tui --list` — lists batteries, exits
- `./target/release/battery-tui --help`
- A short pty run to confirm it samples and exits cleanly: `script -qec "./target/release/battery-tui -i 1" /dev/null` (Ctrl-C to stop).

No test suite exists yet.

## Toolchain constraint

The system `cargo` (apt, 1.75) is too old: latest `clap`/`ratatui` transitive deps require edition2024 / rustc ≥1.85. Rustup-installed stable (1.96+) is used instead. This is why `clap` was dropped in favour of a hand-rolled arg parser (`Cli::parse` in `main.rs`) — keep new dependencies compatible with the rustup toolchain, and prefer not to re-add heavy dep trees.

## Architecture

Single-threaded, no async, no background daemon. One event loop drives both input and sampling.

- `main.rs` — CLI parsing (manual, no clap), terminal setup/teardown via a `TerminalGuard` (Drop restores the terminal even on panic), and the `run` loop. Redraw cadence (250ms) is deliberately decoupled from the sample interval so the UI stays responsive; sampling happens on its own schedule inside `App::on_tick`.
- `battery.rs` — the only code that touches sysfs. `read()` returns a `BatteryInfo`. **Key detail:** it supports both sysfs families — `energy_*`/`power_now` (µWh/µW) and `charge_*`/`current_now` (µAh/µA). Power and health are *derived* when not exposed directly (e.g. power = current × voltage). Every field is optional and guarded; missing fields surface as `n/a`, never a crash. `info.raw` holds every readable scalar field for the Details view.
- `history.rs` — `History` is a capped `VecDeque<Sample>` ring buffer (in-memory only). Provides point series for charts and `Stats` aggregates (energy is integrated as Σ power·dt).
- `app.rs` — `App` owns current `BatteryInfo`, the `History`, selected `Tab`, pause/interval state. `sample()` reads the battery and pushes to history unless paused.
- `ui/` — pure rendering, no state mutation. `mod.rs` lays out header tabs + body + footer and dispatches per `Tab`; `live.rs`, `charts.rs`, `details.rs` are the three views.

## Conventions that matter

- sysfs reads are tolerant by design: add new fields with the `read_u64`/`read_string` helpers and treat absence as normal, not an error.
- ratatui widgets borrow their text. Helper fns that build `Line`/`Span` from `format!` results must take owned `String` (returning `Line<'static>`), not `&str` — passing a `&format!(...)` temporary won't compile.
- When adding data worth showing, surface it in a view (e.g. the Live `Device` panel) rather than leaving struct fields unread — dead-field warnings are treated as a signal to either display or remove.

## Releases & CI

Repo: `github.com/devsomelife/somelife-batteryui` (`gh` authed as `devsomelife`). Default branch `main`.

- `.github/workflows/ci.yml` — runs `fmt --check`, `clippy -D warnings`, `cargo build --release` on push to `main` and PRs.
- `.github/workflows/release.yml` — triggered by a `vX.Y.Z` tag. Builds a **static musl** binary (`x86_64-unknown-linux-musl`) and a `.deb` (via `cargo deb`), then uploads both to a GitHub Release.

Cutting a release: bump `version` in `Cargo.toml`, commit, then `git tag vX.Y.Z && git push origin vX.Y.Z`. The workflow does the rest.

The `.deb` is configured under `[package.metadata.deb]` in `Cargo.toml`. `Cargo.lock` is committed (this is a binary, not a library) for reproducible builds. Releases are x86-64 only — add an `aarch64` target to the release matrix if ARM is needed.
