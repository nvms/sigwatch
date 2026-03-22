use crate::app::{App, Panel};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.active_panel, Panel::Scanner);
    let flat = app.flat_scan_addresses();

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let total: usize = app.scan_results.iter().map(|r| r.addresses.len()).sum();
    let block = Block::default()
        .title(format!(" scan results ({total}) "))
        .borders(Borders::ALL)
        .border_style(border_style);

    if flat.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let hint = ratatui::widgets::Paragraph::new("no scans yet")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    }

    let items: Vec<ListItem> = flat
        .iter()
        .enumerate()
        .map(|(i, (scan_idx, addr))| {
            let selected = is_active && i == app.selected_index;
            let prefix = if selected { "> " } else { "  " };
            let style = if selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("0x{addr:X}"), style),
                Span::styled(
                    format!("  [scan {scan_idx}]"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
