//! Details view: every raw sysfs field as a key/value table.

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let Some(info) = &app.info else { return };

    let rows: Vec<Row> = info
        .raw
        .iter()
        .map(|(k, v)| {
            Row::new(vec![
                Cell::from(k.clone())
                    .style(Style::default().fg(Color::Cyan)),
                Cell::from(v.clone()).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let title = format!(" Raw sysfs fields · {} ({} entries) ", info.name, rows.len());

    let table = Table::new(rows, [Constraint::Length(24), Constraint::Min(0)])
        .header(
            Row::new(vec![Cell::from("field"), Cell::from("value")]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(2);

    f.render_widget(table, area);
}
