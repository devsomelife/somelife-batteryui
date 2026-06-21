//! History view: charge-over-time line chart, power line chart, and aggregates.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.history.len() < 2 {
        let p = Paragraph::new("Collecting history… charts appear after a few samples.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" History "));
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // charge chart
            Constraint::Percentage(45), // power chart
            Constraint::Min(4),         // stats
        ])
        .split(area);

    draw_charge_chart(f, app, chunks[0]);
    draw_power_chart(f, app, chunks[1]);
    draw_stats(f, app, chunks[2]);
}

fn draw_charge_chart(f: &mut Frame, app: &App, area: Rect) {
    let points = app.history.charge_points();
    let x_max = app.history.elapsed_s().max(1.0);

    let dataset = Dataset::default()
        .name("charge %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&points);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Charge % over session "),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, x_max])
                .labels(time_labels(x_max)),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, 100.0])
                .labels(vec![Span::raw("0"), Span::raw("50"), Span::raw("100")]),
        );
    f.render_widget(chart, area);
}

fn draw_power_chart(f: &mut Frame, app: &App, area: Rect) {
    let points = app.history.power_points();
    let x_max = app.history.elapsed_s().max(1.0);
    let y_max = points
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0_f64, f64::max)
        .max(1.0)
        * 1.1;

    let dataset = Dataset::default()
        .name("power W")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&points);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Power draw (W) over session "),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, x_max])
                .labels(time_labels(x_max)),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.1}", y_max / 2.0)),
                    Span::raw(format!("{y_max:.1}")),
                ]),
        );
    f.render_widget(chart, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let s = app.history.stats();
    let line = Line::from(vec![
        stat("samples", s.count.to_string()),
        stat("avg", format!("{:.2} W", s.avg_w)),
        stat("min", format!("{:.2} W", s.min_w)),
        stat("max", format!("{:.2} W", s.max_w)),
        stat("energy", format!("{:.3} Wh", s.energy_wh)),
        stat("span", fmt_secs(app.history.elapsed_s())),
    ]);
    let p = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Session aggregates "),
    );
    f.render_widget(p, area);
}

fn stat(label: &str, val: String) -> Span<'static> {
    Span::styled(
        format!("  {label}: {val} "),
        Style::default().add_modifier(Modifier::BOLD),
    )
}

fn time_labels(x_max: f64) -> Vec<Span<'static>> {
    vec![
        Span::raw("0"),
        Span::raw(fmt_secs(x_max / 2.0)),
        Span::raw(fmt_secs(x_max)),
    ]
}

fn fmt_secs(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}m{s:02}s")
}
