use crate::app::{App, InputMode};
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
            let content = if app.status_message.is_empty() {
                Line::from(vec![
                    Span::styled(
                        format!("pid:{} ", app.process.pid()),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{}ms ", app.poll_rate_ms),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "q:quit  ::cmd  tab:panel  j/k:nav",
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(Span::styled(
                    &app.status_message,
                    Style::default().fg(Color::Yellow),
                ))
            };

            let status = Paragraph::new(content).block(block);
            frame.render_widget(status, area);
        }
    }
}
