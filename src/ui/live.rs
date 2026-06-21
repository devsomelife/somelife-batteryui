//! Live view: charge gauge, key stats, and a recent power-draw sparkline.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

use crate::app::App;
use crate::battery::{format_duration, BatteryInfo};
use crate::ui::charge_color;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let Some(info) = &app.info else { return };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // gauge
            Constraint::Min(6),    // stats
            Constraint::Length(7), // sparkline
        ])
        .split(area);

    // Side-by-side: live status (left) and static device info (right).
    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    draw_gauge(f, info, chunks[0]);
    draw_stats(f, info, mid[0]);
    draw_device(f, info, mid[1]);
    draw_sparkline(f, app, chunks[2]);
}

fn draw_gauge(f: &mut Frame, info: &BatteryInfo, area: Rect) {
    let pct = info.capacity_pct.unwrap_or(0.0);
    let ratio = (pct / 100.0).clamp(0.0, 1.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Charge "))
        .gauge_style(Style::default().fg(charge_color(pct)))
        .ratio(ratio)
        .label(format!("{pct:.0}%"));
    f.render_widget(gauge, area);
}

fn draw_stats(f: &mut Frame, info: &BatteryInfo, area: Rect) {
    let (state_icon, state_color) = if info.is_charging() {
        ("⚡ Charging", Color::Green)
    } else if info.is_discharging() {
        ("▼ Discharging", Color::Yellow)
    } else {
        ("● ", Color::Gray)
    };

    let power = info
        .power_w
        .map(|w| format!("{w:.2} W"))
        .unwrap_or_else(|| "n/a".into());
    let voltage = info
        .voltage_v
        .map(|v| format!("{v:.2} V"))
        .unwrap_or_else(|| "n/a".into());
    let health = info
        .health_pct
        .map(|h| format!("{h:.1} %"))
        .unwrap_or_else(|| "n/a".into());
    let remaining = info
        .time_remaining_s
        .map(format_duration)
        .unwrap_or_else(|| "n/a".into());
    let ac = match info.ac_online {
        Some(true) => "plugged in",
        Some(false) => "on battery",
        None => "n/a",
    };
    let remaining_label = if info.is_charging() {
        "Time to full"
    } else {
        "Time to empty"
    };

    let lines = vec![
        kv(
            "Status",
            format!("{state_icon} ({})", info.status),
            state_color,
        ),
        kv("Power draw", power, Color::White),
        kv("Voltage", voltage, Color::White),
        kv(remaining_label, remaining, Color::White),
        kv("Health", health, Color::White),
        kv("AC adapter", ac.to_string(), Color::White),
        kv(
            "Capacity level",
            info.capacity_level.clone().unwrap_or_else(|| "n/a".into()),
            Color::White,
        ),
    ];

    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(p, area);
}

fn draw_device(f: &mut Frame, info: &BatteryInfo, area: Rect) {
    let na = || "n/a".to_string();
    let cycles = info.cycle_count.map(|c| c.to_string()).unwrap_or_else(na);

    let lines = vec![
        kv(
            "Model",
            info.model_name.clone().unwrap_or_else(na),
            Color::White,
        ),
        kv(
            "Manufacturer",
            info.manufacturer.clone().unwrap_or_else(na),
            Color::White,
        ),
        kv(
            "Technology",
            info.technology.clone().unwrap_or_else(na),
            Color::White,
        ),
        kv("Cycle count", cycles, Color::White),
        kv(
            "Serial",
            info.serial_number.clone().unwrap_or_else(na),
            Color::White,
        ),
    ];

    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Device "));
    f.render_widget(p, area);
}

fn draw_sparkline(f: &mut Frame, app: &App, area: Rect) {
    let data = app
        .history
        .power_sparkline(area.width.saturating_sub(2) as usize);
    let title = if data.is_empty() {
        " Power draw (collecting…) ".to_string()
    } else {
        " Power draw (recent, mW) ".to_string()
    };
    let spark = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Cyan))
        .data(&data);
    f.render_widget(spark, area);
}

fn kv(key: &str, val: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key:<16}"),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(val, Style::default().fg(color)),
    ])
}
