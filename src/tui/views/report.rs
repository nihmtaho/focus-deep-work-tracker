use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::display::format::format_duration;
use crate::tui::app::{App, TimeWindow};

pub fn render(frame: &mut Frame, app: &App, window: &TimeWindow, selected_window: usize) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(3), // tab bar
            Constraint::Min(5),    // table
            Constraint::Length(2), // help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(" Report ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    // Tab bar
    let tabs = ["1:Today", "2:This Week", "3:Last 7 Days"];
    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let is_selected = i == selected_window;
            let span = if is_selected {
                Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {} ", label), Style::default().fg(Color::White))
            };
            vec![span, Span::raw("  ")]
        })
        .collect();
    let tab_bar = Paragraph::new(Line::from(tab_spans)).alignment(Alignment::Left);
    frame.render_widget(tab_bar, chunks[1]);

    // Window label
    let window_label = match window {
        TimeWindow::Today => "Today",
        TimeWindow::CurrentWeek => "This Week (Mon–now)",
        TimeWindow::Last7Days => "Last 7 Days",
    };

    // Table
    let header_cells = ["Tag", "Time"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let total_secs: i64 = app.report_rows.iter().map(|(_, s)| s).sum();

    let mut rows: Vec<Row> = app
        .report_rows
        .iter()
        .map(|(tag, secs)| {
            let label = tag.as_deref().unwrap_or("untagged").to_string();
            let dur = format_duration(chrono::Duration::seconds(*secs));
            Row::new(vec![Cell::from(label), Cell::from(dur)])
        })
        .collect();

    if !app.report_rows.is_empty() {
        rows.push(
            Row::new(vec![
                Cell::from("TOTAL").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(format_duration(chrono::Duration::seconds(total_secs)))
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            ])
            .top_margin(1),
        );
    }

    let col_widths = [Constraint::Min(20), Constraint::Length(15)];

    let empty_msg = if app.report_rows.is_empty() {
        format!("No sessions recorded for {}.", window_label)
    } else {
        String::new()
    };

    if app.report_rows.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            empty_msg,
            Style::default().fg(Color::DarkGray),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(format!(" {} ", window_label)),
        );
        frame.render_widget(empty, chunks[2]);
    } else {
        let table = Table::new(rows, col_widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(format!(" {} ", window_label)),
        );
        frame.render_widget(table, chunks[2]);
    }

    // Help
    let help = Paragraph::new(" [←→] Switch window  [1/2/3] Jump  [Q/Esc] Back ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}
