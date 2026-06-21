//! Application state shared across the UI and event loop.

use std::time::{Duration, Instant};

use crate::battery::{self, BatteryInfo};
use crate::history::History;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Live,
    History,
    Details,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Live, Tab::History, Tab::Details];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Live => "Live",
            Tab::History => "History",
            Tab::Details => "Details",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Tab::Live => 0,
            Tab::History => 1,
            Tab::Details => 2,
        }
    }
}

pub struct App {
    /// Battery name to read, or None to auto-detect.
    battery_name: Option<String>,
    pub info: Option<BatteryInfo>,
    pub history: History,
    pub tab: Tab,
    pub paused: bool,
    pub interval: Duration,
    pub should_quit: bool,
    /// Set when reading the battery fails or no battery is present.
    pub message: Option<String>,
    last_sample: Instant,
}

impl App {
    pub fn new(battery_name: Option<String>, interval: Duration, history_cap: usize) -> Self {
        let mut app = App {
            battery_name,
            info: None,
            history: History::new(history_cap),
            tab: Tab::Live,
            paused: false,
            interval,
            should_quit: false,
            message: None,
            last_sample: Instant::now(),
        };
        app.sample(0.0);
        app
    }

    /// Reads the battery once and pushes to history (unless paused).
    pub fn sample(&mut self, dt_s: f64) {
        match battery::read(self.battery_name.as_deref()) {
            Ok(Some(info)) => {
                if !self.paused {
                    self.history.push(&info, dt_s);
                }
                self.info = Some(info);
                self.message = None;
            }
            Ok(None) => {
                self.info = None;
                self.message = Some("No battery detected on this system.".into());
            }
            Err(e) => {
                self.info = None;
                self.message = Some(e.to_string());
            }
        }
    }

    /// Called on each tick; samples if the interval has elapsed.
    pub fn on_tick(&mut self) {
        let dt = self.last_sample.elapsed();
        if dt >= self.interval {
            let dt_s = dt.as_secs_f64();
            self.last_sample = Instant::now();
            self.sample(dt_s);
        }
    }

    pub fn next_tab(&mut self) {
        let i = (self.tab.index() + 1) % Tab::ALL.len();
        self.tab = Tab::ALL[i];
    }

    pub fn select_tab(&mut self, i: usize) {
        if let Some(&t) = Tab::ALL.get(i) {
            self.tab = t;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn reset_history(&mut self) {
        self.history.clear();
    }

    pub fn adjust_interval(&mut self, delta_s: i64) {
        let cur = self.interval.as_secs() as i64;
        let next = (cur + delta_s).clamp(1, 60);
        self.interval = Duration::from_secs(next as u64);
    }
}
