use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::display::format::format_duration;
use crate::tui::app::truncate_to;
use crate::tui::app::{App, LOG_PAGE_SIZE};

pub fn render(frame: &mut Frame, app: &App, page: usize) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(5),    // table
            Constraint::Length(2), // pagination
            Constraint::Length(2), // help
        ])
        .split(area);

    // Title
    let total = app.log_entries.len();
    let title_text = format!(
        " Session Log — {} total, page {}/{} ",
        total,
        page + 1,
        app.log_total_pages
    );
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    // Table
    let header_cells = ["Date", "Task", "Tag", "Duration"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let entries = app.log_page_entries(page);
    let rows: Vec<Row> = entries
        .iter()
        .map(|s| {
            let date = s.start_time.format("%Y-%m-%d %H:%M").to_string();
            let task = truncate_to(&s.task, 30);
            let tag = s.tag.as_deref().unwrap_or("—").to_string();
            let duration = s
                .duration()
                .map(format_duration)
                .unwrap_or_else(|| "—".to_string());
            Row::new(vec![
                Cell::from(date),
                Cell::from(task),
                Cell::from(tag),
                Cell::from(duration),
            ])
        })
        .collect();

    let col_widths = [
        Constraint::Length(17),
        Constraint::Min(20),
        Constraint::Length(15),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, col_widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Completed Sessions "),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_widget(table, chunks[1]);

    // Pagination info
    let start_item = page * LOG_PAGE_SIZE + 1;
    let end_item = (start_item + entries.len()).saturating_sub(1);
    let page_info = if total == 0 {
        "No sessions recorded.".to_string()
    } else {
        format!("Showing {} – {} of {}", start_item, end_item, total)
    };
    let pagination = Paragraph::new(Line::from(vec![Span::styled(
        page_info,
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(pagination, chunks[2]);

    // Help
    let help = Paragraph::new(" [←/P] Prev  [→/N] Next  [Q/Esc] Back ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}
