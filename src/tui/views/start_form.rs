use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, InputField};

pub fn render(frame: &mut Frame, app: &App, task: &str, tag: &str, active_field: &InputField) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new(" Start New Session ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    // Task input
    let task_style = if *active_field == InputField::Task {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let task_text = if task.is_empty() && *active_field != InputField::Task {
        Span::styled("(empty)", Style::default().fg(Color::DarkGray))
    } else {
        let cursor = if *active_field == InputField::Task {
            "█"
        } else {
            ""
        };
        Span::styled(format!("{}{}", task, cursor), task_style)
    };
    let task_block = Paragraph::new(Line::from(vec![task_text])).block(
        Block::default()
            .title(" Task (required) ")
            .borders(Borders::ALL)
            .border_style(if *active_field == InputField::Task {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Blue)
            }),
    );
    frame.render_widget(task_block, chunks[1]);

    // Tag input
    let tag_style = if *active_field == InputField::Tag {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let tag_text = if tag.is_empty() && *active_field != InputField::Tag {
        Span::styled("(optional)", Style::default().fg(Color::DarkGray))
    } else {
        let cursor = if *active_field == InputField::Tag {
            "█"
        } else {
            ""
        };
        Span::styled(format!("{}{}", tag, cursor), tag_style)
    };
    let tag_block = Paragraph::new(Line::from(vec![tag_text])).block(
        Block::default()
            .title(" Tag (optional) ")
            .borders(Borders::ALL)
            .border_style(if *active_field == InputField::Tag {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Blue)
            }),
    );
    frame.render_widget(tag_block, chunks[2]);

    let help = Paragraph::new(" [Tab] Switch field  [Enter] Submit  [Esc] Cancel ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[4]);

    // Message overlay if any
    if let Some(msg) = &app.message {
        super::dashboard::render_message_overlay_pub(frame, app, msg);
    }
}
