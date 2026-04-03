use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(5),    // content
            Constraint::Length(2), // help bar
        ])
        .split(area);

    let title = Paragraph::new(" Settings ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    let vim_label = if app.config.vim_mode {
        "Vim Mode: [ ON  ]"
    } else {
        "Vim Mode: [ OFF ]"
    };
    let vim_style = if app.config.vim_mode {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let content = Paragraph::new(vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(vim_label, vim_style)),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" Preferences "),
    );
    frame.render_widget(content, chunks[1]);

    let help = Paragraph::new(" [V] Toggle Vim Mode ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);

    // Message overlay if present
    if let Some(msg) = &app.message {
        crate::tui::views::dashboard::render_message_overlay_pub(frame, app, msg);
    }
}
