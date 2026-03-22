use crate::app::App;
use crate::widgets::{memory, scanner, status};
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_area);

    let left_panel = panels[0];
    let right_panel = panels[1];

    memory::render(frame, app, left_panel);
    scanner::render(frame, app, right_panel);
    status::render(frame, app, status_area);
}
