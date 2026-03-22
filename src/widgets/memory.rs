use crate::app::{App, Panel};
use crate::display;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, HighlightSpacing, Row, Table};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_active = matches!(app.active_panel, Panel::Watches);

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(format!(" watches ({}) ", app.watch_list.len()))
        .borders(Borders::ALL)
        .border_style(border_style);

    if app.watch_list.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let hint = ratatui::widgets::Paragraph::new("no watches - press : to enter commands")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    }

    let header = Row::new(vec!["#", "label", "value", "fmt"])
        .style(Style::default().fg(Color::DarkGray))
        .bottom_margin(0);

    let rows: Vec<Row> = app
        .watch_list
        .watches
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let value_str = display::format_bytes(&w.current, w.display_format());
            let style = if w.changed {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("{i}"),
                w.label.clone(),
                value_str,
                format!("{}", w.display_format()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Min(16),
        Constraint::Min(24),
        Constraint::Length(8),
    ];

    let highlight = if is_active {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(highlight)
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(table, area, &mut app.watch_table_state);
}
