use crate::app::{App, Panel};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_active = matches!(app.active_panel, Panel::Scanner);

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let session = match &app.scan {
        Some(s) => s,
        None => {
            let block = Block::default()
                .title(" scanner ")
                .borders(Borders::ALL)
                .border_style(border_style);
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  type :scan <value> to search process memory",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  examples:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "    :scan 75.0     search for float",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "    :scan 42       search for integer",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "    :scan DE AD    search for hex bytes",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            frame.render_widget(Paragraph::new(lines), inner);
            return;
        }
    };

    let count = session.candidates.len();
    let vtype = session.value_type.label();
    let steps = session.history.len();

    let title = format!(" {count} candidates ({vtype}) [{steps} step(s)] ");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    if session.candidates.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  no candidates left",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  :reset to start a new scan",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let items: Vec<ListItem> = session
        .candidates
        .iter()
        .map(|c| {
            let value = session.format_value(c);
            ListItem::new(Line::from(vec![
                Span::styled(format!("0x{:X}", c.address), Style::default()),
                Span::styled("  =  ", Style::default().fg(Color::DarkGray)),
                Span::styled(value, Style::default().fg(Color::Cyan)),
            ]))
        })
        .collect();

    let highlight = if is_active {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight)
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.scanner_list_state);
}
