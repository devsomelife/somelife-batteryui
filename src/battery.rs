//! Reads battery and AC adapter information from `/sys/class/power_supply`.
//!
//! Handles both the `energy_*`/`power_now` (µWh/µW) family and the
//! `charge_*`/`current_now` (µAh/µA) family, deriving power and health
//! tolerantly when fields are missing.

use std::fs;
use std::path::{Path, PathBuf};

const POWER_SUPPLY: &str = "/sys/class/power_supply";

/// A snapshot of a single battery plus the system AC state.
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,

    pub status: String,
    pub capacity_pct: Option<f64>,
    pub capacity_level: Option<String>,

    /// Derived instantaneous power draw in watts (always positive magnitude).
    pub power_w: Option<f64>,
    /// Derived battery health: full / full_design as a percentage.
    pub health_pct: Option<f64>,
    /// Derived time remaining in seconds (to empty when discharging, to full
    /// when charging).
    pub time_remaining_s: Option<f64>,

    pub voltage_v: Option<f64>,
    pub cycle_count: Option<u64>,

    pub technology: Option<String>,
    pub manufacturer: Option<String>,
    pub model_name: Option<String>,
    pub serial_number: Option<String>,

    pub ac_online: Option<bool>,

    /// Every raw key/value pair read from the battery directory, for the
    /// Details view. Sorted by key.
    pub raw: Vec<(String, String)>,
}

impl BatteryInfo {
    pub fn is_charging(&self) -> bool {
        self.status.eq_ignore_ascii_case("charging")
    }

    pub fn is_discharging(&self) -> bool {
        self.status.eq_ignore_ascii_case("discharging")
    }
}

/// Returns the names of all `Battery`-type supplies, sorted.
pub fn list_batteries() -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(POWER_SUPPLY) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if read_string(&path.join("type"))
            .map(|t| t.eq_ignore_ascii_case("battery"))
            .unwrap_or(false)
        {
            if let Some(name) = entry.file_name().to_str() {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out
}

/// Reads a single battery by name. Pass `None` to auto-detect the first
/// battery-type supply. Returns `Ok(None)` when no battery exists at all.
pub fn read(name: Option<&str>) -> anyhow::Result<Option<BatteryInfo>> {
    let name = match name {
        Some(n) => n.to_string(),
        None => match list_batteries().into_iter().next() {
            Some(n) => n,
            None => return Ok(None),
        },
    };

    let dir = Path::new(POWER_SUPPLY).join(&name);
    if !dir.is_dir() {
        let available = list_batteries();
        anyhow::bail!(
            "battery '{name}' not found. Available batteries: {}",
            if available.is_empty() {
                "(none)".to_string()
            } else {
                available.join(", ")
            }
        );
    }

    let status = read_string(&dir.join("status")).unwrap_or_else(|| "Unknown".into());
    let capacity_pct = read_u64(&dir.join("capacity")).map(|v| v as f64);
    let capacity_level = read_string(&dir.join("capacity_level"));

    let voltage_uv = read_u64(&dir.join("voltage_now"));
    let voltage_v = voltage_uv.map(|v| v as f64 / 1e6);

    // Power: prefer power_now (µW); else current_now (µA) * voltage_now (µV).
    let power_w = match read_u64(&dir.join("power_now")) {
        Some(uw) => Some(uw as f64 / 1e6),
        None => match (read_u64(&dir.join("current_now")), voltage_uv) {
            (Some(ua), Some(uv)) => Some((ua as f64 / 1e6) * (uv as f64 / 1e6)),
            _ => None,
        },
    };

    // Charge/energy families. energy_* is in µWh, charge_* in µAh.
    let energy_now = read_u64(&dir.join("energy_now"));
    let energy_full = read_u64(&dir.join("energy_full"));
    let energy_full_design = read_u64(&dir.join("energy_full_design"));
    let charge_now = read_u64(&dir.join("charge_now"));
    let charge_full = read_u64(&dir.join("charge_full"));
    let charge_full_design = read_u64(&dir.join("charge_full_design"));

    let (now, full, full_design) = if energy_full.is_some() || energy_now.is_some() {
        (energy_now, energy_full, energy_full_design)
    } else {
        (charge_now, charge_full, charge_full_design)
    };

    let health_pct = match (full, full_design) {
        (Some(f), Some(d)) if d > 0 => Some(f as f64 / d as f64 * 100.0),
        _ => None,
    };

    // Time remaining. For the charge_* family we have µAh and derive watts from
    // µA, so use current_now (µA) directly against charge (µAh) -> hours.
    // For the energy_* family, use power_now (µW) against energy (µWh) -> hours.
    let time_remaining_s = compute_time_remaining(&dir, &status, now, full, energy_now.is_some());

    let cycle_count = read_u64(&dir.join("cycle_count"));

    let mut raw = read_all_fields(&dir);
    raw.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Some(BatteryInfo {
        name,
        status,
        capacity_pct,
        capacity_level,
        power_w,
        health_pct,
        time_remaining_s,
        voltage_v,
        cycle_count,
        technology: read_string(&dir.join("technology")),
        manufacturer: read_string(&dir.join("manufacturer")),
        model_name: read_string(&dir.join("model_name")),
        serial_number: read_string(&dir.join("serial_number")),
        ac_online: read_ac_online(),
        raw,
    }))
}

/// Computes seconds until empty (discharging) or full (charging).
/// `is_energy` selects the unit family. Returns `None` when not derivable.
fn compute_time_remaining(
    dir: &Path,
    status: &str,
    now: Option<u64>,
    full: Option<u64>,
    is_energy: bool,
) -> Option<f64> {
    let charging = status.eq_ignore_ascii_case("charging");
    let discharging = status.eq_ignore_ascii_case("discharging");
    if !charging && !discharging {
        return None;
    }
    let now = now?;

    // rate is in the per-hour unit matching `now` (µWh/h via µW, or µAh/h via µA).
    let rate = if is_energy {
        read_u64(&dir.join("power_now"))?
    } else {
        read_u64(&dir.join("current_now"))?
    };
    if rate == 0 {
        return None;
    }

    let remaining_units = if charging {
        full?.saturating_sub(now)
    } else {
        now
    };
    Some(remaining_units as f64 / rate as f64 * 3600.0)
}

/// Reads `online` from the first `Mains`-type supply that is present.
fn read_ac_online() -> Option<bool> {
    let entries = fs::read_dir(POWER_SUPPLY).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if read_string(&path.join("type"))
            .map(|t| t.eq_ignore_ascii_case("mains"))
            .unwrap_or(false)
        {
            if let Some(v) = read_u64(&path.join("online")) {
                return Some(v != 0);
            }
        }
    }
    None
}

/// Reads every readable scalar field in a power-supply directory.
fn read_all_fields(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Only plain files; skip subdirs (device, power, hwmon, subsystem, ...).
        if !path.is_file() {
            continue;
        }
        if let (Some(name), Some(val)) = (
            path.file_name().and_then(|n| n.to_str()),
            read_string(&path),
        ) {
            if !val.is_empty() {
                out.push((name.to_string(), val));
            }
        }
    }
    out
}

fn read_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_u64(path: &Path) -> Option<u64> {
    read_string(path).and_then(|s| s.parse::<u64>().ok())
}

/// Formats a duration in seconds as `Hh Mm`.
pub fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "n/a".into();
    }
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    format!("{h}h {m:02}m")
}

// Suppress unused-path-construction warnings on platforms without these files.
#[allow(dead_code)]
fn _unused() -> PathBuf {
    PathBuf::from(POWER_SUPPLY)
}
