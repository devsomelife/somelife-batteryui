//! battery-tui: a terminal battery monitor for Linux, reading sysfs.

mod app;
mod battery;
mod history;
mod ui;

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

#[derive(Debug)]
struct Cli {
    interval: u64,
    battery: Option<String>,
    history: usize,
    list: bool,
}

const USAGE: &str = "\
battery-tui — a 100% terminal battery monitor for Linux (sysfs-based)

USAGE:
    battery-tui [OPTIONS]

OPTIONS:
    -i, --interval <SECS>   Sample interval in seconds, 1-60 (default: 2)
    -b, --battery <NAME>    Battery name, e.g. BAT0 (default: auto-detect)
        --history <N>       Samples kept in the in-memory ring buffer (default: 600)
        --list              List available batteries and exit
    -h, --help              Print help
    -V, --version           Print version";

impl Cli {
    /// Minimal hand-rolled arg parser (no clap, to stay compatible with the
    /// system's older Cargo). Handles `--flag value` and `--flag=value`.
    fn parse() -> Result<Cli> {
        let mut cli = Cli {
            interval: 2,
            battery: None,
            history: 600,
            list: false,
        };

        let mut args = std::env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            // Split `--key=value` into key and inline value.
            let (key, inline) = match arg.split_once('=') {
                Some((k, v)) => (k.to_string(), Some(v.to_string())),
                None => (arg.clone(), None),
            };

            let mut take_value = |key: &str| -> Result<String> {
                if let Some(v) = inline.clone() {
                    return Ok(v);
                }
                match args.next() {
                    Some(v) => Ok(v),
                    None => bail!("missing value for {key}"),
                }
            };

            match key.as_str() {
                "-h" | "--help" => {
                    println!("{USAGE}");
                    std::process::exit(0);
                }
                "-V" | "--version" => {
                    println!("battery-tui {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--list" => cli.list = true,
                "-i" | "--interval" => {
                    let v: u64 = take_value(&key)?
                        .parse()
                        .map_err(|_| anyhow::anyhow!("invalid --interval"))?;
                    if !(1..=60).contains(&v) {
                        bail!("--interval must be between 1 and 60");
                    }
                    cli.interval = v;
                }
                "-b" | "--battery" => cli.battery = Some(take_value(&key)?),
                "--history" => {
                    cli.history = take_value(&key)?
                        .parse()
                        .map_err(|_| anyhow::anyhow!("invalid --history"))?;
                }
                other => bail!("unknown argument: {other}\n\n{USAGE}"),
            }
        }
        Ok(cli)
    }
}

/// Restores the terminal on drop, even if the app panics.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse()?;

    if cli.list {
        let batteries = battery::list_batteries();
        if batteries.is_empty() {
            println!("No batteries found under /sys/class/power_supply.");
        } else {
            println!("Available batteries:");
            for b in batteries {
                println!("  {b}");
            }
        }
        return Ok(());
    }

    let mut terminal = setup_terminal()?;
    let _guard = TerminalGuard;

    let mut app = App::new(
        cli.battery,
        Duration::from_secs(cli.interval),
        cli.history.max(2),
    );

    let res = run(&mut terminal, &mut app);

    // Guard restores the terminal on drop; surface any loop error afterwards.
    drop(_guard);
    res
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    // Redraw cadence is independent of the sample interval so the UI stays
    // responsive; sampling happens on its own schedule inside on_tick.
    let draw_interval = Duration::from_millis(250);
    let mut last_draw = Instant::now() - draw_interval;

    while !app.should_quit {
        if last_draw.elapsed() >= draw_interval {
            terminal.draw(|f| ui::draw(f, app))?;
            last_draw = Instant::now();
        }

        // Wait for input up to the next draw deadline.
        let timeout = draw_interval
            .checked_sub(last_draw.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key.code, key.modifiers);
                }
            }
        }

        app.on_tick();
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Tab | KeyCode::Right => app.next_tab(),
        KeyCode::Char('1') => app.select_tab(0),
        KeyCode::Char('2') => app.select_tab(1),
        KeyCode::Char('3') => app.select_tab(2),
        KeyCode::Char('p') => app.toggle_pause(),
        KeyCode::Char('r') => app.reset_history(),
        KeyCode::Char('+') | KeyCode::Char('=') => app.adjust_interval(1),
        KeyCode::Char('-') | KeyCode::Char('_') => app.adjust_interval(-1),
        _ => {}
    }
}
