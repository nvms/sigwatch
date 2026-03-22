use crate::app::{App, Panel};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.active_panel, Panel::Scanner);

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(format!(" scan results ({}) ", app.scan_results.len()))
        .borders(Borders::ALL)
        .border_style(border_style);

    if app.scan_results.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let hint = ratatui::widgets::Paragraph::new("no scans yet")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .scan_results
        .iter()
        .enumerate()
        .flat_map(|(i, result)| {
            let mut items = vec![ListItem::new(Line::from(vec![
                Span::styled(format!("[{i}] "), Style::default().fg(Color::DarkGray)),
                Span::styled(&result.pattern, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" ({} hits)", result.addresses.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))];

            for addr in result.addresses.iter().take(10) {
                items.push(ListItem::new(format!("  0x{addr:X}")));
            }
            if result.addresses.len() > 10 {
                items.push(ListItem::new(format!(
                    "  ... +{} more",
                    result.addresses.len() - 10
                )));
            }
            items
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
