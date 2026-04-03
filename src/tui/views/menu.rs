use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;

const MENU_ITEMS: &[&str] = &[
    "1. Start a new session",
    "2. Stop current session",
    "3. View session log",
    "4. View report",
    "5. Back to dashboard",
];

pub fn render(frame: &mut Frame, app: &App, selected: usize) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new(" Main Menu ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == selected {
                ListItem::new(Line::from(vec![Span::styled(
                    format!(" > {} ", label),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]))
            } else {
                ListItem::new(Line::from(vec![Span::raw(format!("   {} ", label))]))
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" Navigate "),
    );
    frame.render_widget(list, chunks[1]);

    let help = Paragraph::new(" [↑↓] Navigate  [Enter] Select  [Q/Esc] Back ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);

    // Message overlay if any
    if let Some(msg) = &app.message {
        super::dashboard::render_message_overlay_pub(frame, app, msg);
    }
}
