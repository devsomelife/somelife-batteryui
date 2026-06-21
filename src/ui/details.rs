//! Details view: every raw sysfs field as a field/description/value table.

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;

/// How to convert a raw sysfs value into something human-readable.
#[derive(Clone, Copy)]
enum FmtKind {
    /// µAh -> mAh
    MicroAmpHours,
    /// µWh -> Wh
    MicroWattHours,
    /// µV -> V
    MicroVolts,
    /// µA -> A
    MicroAmps,
    /// µW -> W
    MicroWatts,
    /// integer percent, append `%`
    Percent,
    /// show the raw string unchanged
    Passthrough,
}

/// Known fields, in the order they should appear. Anything not listed here is
/// appended afterwards in its original (alphabetical) order.
const ORDER: &[&str] = &[
    "status",
    "capacity",
    "capacity_level",
    "charge_now",
    "charge_full",
    "charge_full_design",
    "energy_now",
    "energy_full",
    "energy_full_design",
    "voltage_now",
    "voltage_min_design",
    "current_now",
    "power_now",
    "cycle_count",
    "technology",
    "manufacturer",
    "model_name",
    "serial_number",
    "type",
    "present",
    "alarm",
];

/// Returns a human description and value format for a known sysfs field.
fn field_meta(name: &str) -> Option<(&'static str, FmtKind)> {
    use FmtKind::*;
    Some(match name {
        "status" => ("Charge state reported by firmware", Passthrough),
        "capacity" => ("Charge level", Percent),
        "capacity_level" => ("Coarse charge level (Low/Normal/Full)", Passthrough),
        "charge_now" => ("Current charge in the pack", MicroAmpHours),
        "charge_full" => ("Charge when last full (real capacity)", MicroAmpHours),
        "charge_full_design" => ("Charge when full as designed", MicroAmpHours),
        "energy_now" => ("Current energy in the pack", MicroWattHours),
        "energy_full" => ("Energy when last full (real capacity)", MicroWattHours),
        "energy_full_design" => ("Energy when full as designed", MicroWattHours),
        "voltage_now" => ("Terminal voltage", MicroVolts),
        "voltage_min_design" => ("Design minimum voltage", MicroVolts),
        "current_now" => ("Current in/out of the pack", MicroAmps),
        "power_now" => ("Power in/out of the pack", MicroWatts),
        "cycle_count" => ("Charge cycles so far", Passthrough),
        "technology" => ("Cell chemistry", Passthrough),
        "manufacturer" => ("Pack manufacturer", Passthrough),
        "model_name" => ("Pack model", Passthrough),
        "serial_number" => ("Pack serial number", Passthrough),
        "type" => ("Power-supply type", Passthrough),
        "present" => ("Battery present (1 = yes)", Passthrough),
        "alarm" => ("Low-charge alarm threshold (0 = off)", Passthrough),
        _ => return None,
    })
}

/// Formats a raw sysfs value per `kind`, falling back to the raw string when it
/// cannot be parsed.
fn humanize(raw: &str, kind: FmtKind) -> String {
    let parsed = raw.parse::<f64>().ok();
    match (kind, parsed) {
        (FmtKind::MicroAmpHours, Some(v)) => format!("{:.0} mAh", v / 1e3),
        (FmtKind::MicroWattHours, Some(v)) => format!("{:.2} Wh", v / 1e6),
        (FmtKind::MicroVolts, Some(v)) => format!("{:.2} V", v / 1e6),
        (FmtKind::MicroAmps, Some(v)) => format!("{:.3} A", v / 1e6),
        (FmtKind::MicroWatts, Some(v)) => format!("{:.2} W", v / 1e6),
        (FmtKind::Percent, Some(v)) => format!("{v:.0}%"),
        _ => raw.to_string(),
    }
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let Some(info) = &app.info else { return };

    // Curated order first, then any remaining fields (except the redundant
    // multi-line `uevent` blob) in their existing alphabetical order.
    let mut ordered: Vec<&(String, String)> = Vec::with_capacity(info.raw.len());
    for key in ORDER {
        if let Some(entry) = info.raw.iter().find(|(k, _)| k == key) {
            ordered.push(entry);
        }
    }
    for entry in &info.raw {
        if entry.0 == "uevent" || ORDER.contains(&entry.0.as_str()) {
            continue;
        }
        ordered.push(entry);
    }

    let rows: Vec<Row> = ordered
        .iter()
        .map(|(k, v)| {
            let (desc, value) = match field_meta(k) {
                Some((d, kind)) => (d.to_string(), humanize(v, kind)),
                None => (String::new(), v.clone()),
            };
            Row::new(vec![
                Cell::from(k.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(value).style(Style::default().fg(Color::White)),
                Cell::from(desc).style(Style::default().fg(Color::Gray)),
            ])
        })
        .collect();

    let title = format!(" Details · {} ({} fields) ", info.name, rows.len());

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(14),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from("field"),
            Cell::from("value"),
            Cell::from("description"),
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
    .column_spacing(2);

    f.render_widget(table, area);
}
