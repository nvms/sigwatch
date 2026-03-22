use crate::app::{App, InputMode, Panel};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    match &app.input_mode {
        InputMode::Command => {
            let input = Paragraph::new(Line::from(vec![
                Span::styled(":", Style::default().fg(Color::Yellow)),
                Span::raw(&app.input_buffer),
            ]))
            .block(block);
            frame.render_widget(input, area);
        }
        InputMode::Normal => {
            let content = if let Some(msg) = app.active_status() {
                Line::from(vec![
                    Span::styled(msg, Style::default().fg(Color::Yellow)),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        hints_for_panel(&app.active_panel),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("pid:{} ", app.process.pid()),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{}ms  ", app.poll_rate_ms),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        hints_for_panel(&app.active_panel),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            };

            let status = Paragraph::new(content).block(block);
            frame.render_widget(status, area);
        }
    }
}

fn hints_for_panel(panel: &Panel) -> &'static str {
    match panel {
        Panel::Watches => "q:quit  ::cmd  tab:panel  j/k:nav  ?:help",
        Panel::Scanner => "q:quit  ::cmd  tab:panel  j/k:nav  enter:pick as type  ?:help",
        Panel::Modules => "q:quit  ::cmd  tab:panel  j/k:nav  ?:help",
    }
}
