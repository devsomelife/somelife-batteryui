# Contributing to battery-tui

Thanks for your interest in contributing! This document explains how to get set up and what we expect from contributions.

## Getting started

`battery-tui` is a Rust project. The system's apt `cargo` may be too old; install a recent stable toolchain via [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
```

Then build and run:

```sh
cargo build
cargo run
```

The app is a TUI and needs a real terminal (it uses the alternate screen). Don't run it through a pipe.

## Before you open a pull request

CI runs the following and fails on any of them, so please run them locally first:

```sh
cargo fmt --check                          # formatting
cargo clippy --all-targets -- -D warnings  # lints (warnings are errors)
cargo build --release                      # it compiles
```

Run `cargo fmt` to auto-fix formatting.

Quick manual smoke test without a battery state change:

```sh
./target/release/battery-tui --list   # lists detected batteries
./target/release/battery-tui          # launch the TUI (q to quit)
```

## Project layout

See [CLAUDE.md](CLAUDE.md) for the architecture overview. In short:

- `src/battery.rs` — the only code that reads `/sys/class/power_supply`. Reads are deliberately tolerant: missing fields are `n/a`, never a crash. Supports both the `energy_*`/`power_now` and `charge_*`/`current_now` sysfs families.
- `src/history.rs` — in-memory ring buffer of samples and session aggregates.
- `src/app.rs` — application state.
- `src/ui/` — rendering only, no state mutation.
- `src/main.rs` — CLI parsing and the event loop.

## Coding conventions

- Keep sysfs reads tolerant: use the `read_u64` / `read_string` helpers and treat absence as normal.
- Avoid adding heavy dependency trees; the project intentionally has a small, modern dep set (`ratatui`, `crossterm`, `anyhow`).
- If you add data worth showing, surface it in a view rather than leaving struct fields unread.

## Commit messages

Follow [git karma](https://karma-runner.github.io/latest/dev/git-commit-msg.html) conventions: a short imperative subject line prefixed with a type (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, ...). No description block is required for small changes.

## Reporting bugs and requesting features

Use the issue templates. For bugs, please include the output of `battery-tui --list` and the contents of `/sys/class/power_supply/BAT*/uevent` where relevant — battery sysfs layouts vary across hardware.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
