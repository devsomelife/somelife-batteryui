//! In-memory ring buffer of battery samples plus session aggregates.

use std::collections::VecDeque;

use crate::battery::BatteryInfo;

#[derive(Debug, Clone)]
pub struct Sample {
    /// Seconds since the session (history) started.
    pub elapsed_s: f64,
    pub charge_pct: f64,
    pub power_w: f64,
    pub voltage_v: f64,
    pub charging: bool,
}

pub struct History {
    samples: VecDeque<Sample>,
    capacity: usize,
    elapsed_s: f64,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
            elapsed_s: 0.0,
        }
    }

    /// Records a sample. `dt_s` is the seconds elapsed since the previous push.
    pub fn push(&mut self, info: &BatteryInfo, dt_s: f64) {
        if !self.samples.is_empty() {
            self.elapsed_s += dt_s;
        }
        let sample = Sample {
            elapsed_s: self.elapsed_s,
            charge_pct: info.capacity_pct.unwrap_or(0.0),
            power_w: info.power_w.unwrap_or(0.0),
            voltage_v: info.voltage_v.unwrap_or(0.0),
            charging: info.is_charging(),
        };
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.elapsed_s = 0.0;
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sample> {
        self.samples.iter()
    }

    /// Charge % points as (elapsed_s, value) for charting.
    pub fn charge_points(&self) -> Vec<(f64, f64)> {
        self.samples
            .iter()
            .map(|s| (s.elapsed_s, s.charge_pct))
            .collect()
    }

    /// Power draw points as (elapsed_s, value) for charting.
    pub fn power_points(&self) -> Vec<(f64, f64)> {
        self.samples
            .iter()
            .map(|s| (s.elapsed_s, s.power_w))
            .collect()
    }

    /// Recent power-draw values, scaled to u64 milliwatts, for a sparkline.
    pub fn power_sparkline(&self, max_points: usize) -> Vec<u64> {
        let skip = self.samples.len().saturating_sub(max_points);
        self.samples
            .iter()
            .skip(skip)
            .map(|s| (s.power_w * 1000.0) as u64)
            .collect()
    }

    pub fn elapsed_s(&self) -> f64 {
        self.elapsed_s
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        if self.samples.is_empty() {
            return s;
        }
        let mut sum = 0.0;
        s.min_w = f64::MAX;
        s.max_w = f64::MIN;
        let mut prev: Option<&Sample> = None;
        for sample in &self.samples {
            sum += sample.power_w;
            s.min_w = s.min_w.min(sample.power_w);
            s.max_w = s.max_w.max(sample.power_w);
            // Integrate energy: power (W) * dt (h) = Wh.
            if let Some(p) = prev {
                let dt_h = (sample.elapsed_s - p.elapsed_s) / 3600.0;
                s.energy_wh += p.power_w * dt_h;
            }
            prev = Some(sample);
        }
        s.avg_w = sum / self.samples.len() as f64;
        s.count = self.samples.len();
        s
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub count: usize,
    pub avg_w: f64,
    pub min_w: f64,
    pub max_w: f64,
    /// Energy that flowed (in or out) over the session, in watt-hours.
    pub energy_wh: f64,
}
