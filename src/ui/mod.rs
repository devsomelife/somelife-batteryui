//! Top-level layout: header tabs, body (per selected tab), footer keybinds.

mod charts;
mod details;
mod live;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, Tab};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header / tabs
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_body(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL.iter().map(|t| Line::from(t.title())).collect();

    let name = app
        .info
        .as_ref()
        .map(|i| i.name.as_str())
        .unwrap_or("Some-BatteryUI");

    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            format!(" Some-BatteryUI · {name} "),
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" ");

    f.render_widget(tabs, area);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    if let Some(msg) = &app.message {
        let p = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Info "));
        f.render_widget(p, area);
        return;
    }

    match app.tab {
        Tab::Live => live::draw(f, app, area),
        Tab::History => charts::draw(f, app, area),
        Tab::Details => details::draw(f, app, area),
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let pause = if app.paused {
        "▶ resume"
    } else {
        "⏸ pause"
    };
    let spans = Line::from(vec![
        key("q"),
        Span::raw(" quit  "),
        key("Tab/1-3"),
        Span::raw(" view  "),
        key("p"),
        Span::raw(format!(" {pause}  ")),
        key("r"),
        Span::raw(" reset  "),
        key("+/-"),
        Span::raw(format!(" interval {}s", app.interval.as_secs())),
    ]);
    f.render_widget(
        Paragraph::new(spans).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn key(k: &str) -> Span<'_> {
    Span::styled(
        k,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

/// Shared color for a charge percentage.
pub fn charge_color(pct: f64) -> Color {
    if pct >= 50.0 {
        Color::Green
    } else if pct >= 20.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}
