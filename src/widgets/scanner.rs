use crate::app::{App, Panel};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
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
        .map(|(scan_idx, addr)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("0x{addr:X}"), Style::default()),
                Span::styled(
                    format!("  [scan {scan_idx}]"),
                    Style::default().fg(Color::DarkGray),
                ),
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
