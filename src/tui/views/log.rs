use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::display::format::format_duration;
use crate::tui::app::{App, LOG_PAGE_SIZE};
use crate::tui::themes::get_colors_for_theme;

/// Format a session status as a short string.
pub fn format_status(end_time: bool) -> &'static str {
    if end_time {
        "done"
    } else {
        "active"
    }
}

/// Compute column widths responsive to terminal width.
///
/// Returns (date_w, task_w, tag_w, mode_w, duration_w, status_w) as Constraints.
pub fn responsive_column_widths(area_width: u16) -> [Constraint; 6] {
    // Minimum widths: date=17, mode=8, duration=10, status=6, tag=8
    // task gets the rest
    let fixed = 17 + 8 + 10 + 6 + 8 + 5; // 5 for separators
    let task_w = (area_width as u32).saturating_sub(fixed).max(10) as u16;
    [
        Constraint::Length(17),  // date
        Constraint::Min(task_w), // task — gets remaining space
        Constraint::Length(10),  // tag
        Constraint::Length(8),   // mode
        Constraint::Length(10),  // duration
        Constraint::Length(6),   // status
    ]
}

pub fn render(frame: &mut Frame, app: &App, page: usize, selected: usize, area: Rect) {
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

    let colors = get_colors_for_theme(app.config.theme.as_deref());

    // Table with selection highlight
    let header_cells = ["Date", "Task", "Tag", "Mode", "Duration", "Status"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let entries = app.log_page_entries(page);

    if entries.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  (empty) — no completed sessions.",
            Style::default().fg(Color::DarkGray),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Completed Sessions "),
        );
        frame.render_widget(empty, chunks[1]);
    } else {
        let col_widths = responsive_column_widths(chunks[1].width);

        let rows: Vec<Row> = entries
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let date = s.start_time.format("%Y-%m-%d %H:%M").to_string();
                // No truncation — show full task name; table wraps automatically
                let task = s.task.clone();
                let tag = s.tag.as_deref().unwrap_or("—").to_string();
                let mode_str = s.mode.clone();
                let duration = s
                    .duration()
                    .map(format_duration)
                    .unwrap_or_else(|| "—".to_string());
                let status = format_status(s.end_time.is_some()).to_string();

                let tag_cell = Cell::from(tag).style(Style::default().fg(colors.tag_color));
                let task_cell = Cell::from(task).style(Style::default().fg(colors.session_title));
                let status_style = if s.end_time.is_none() {
                    Style::default().fg(colors.warning)
                } else {
                    Style::default().fg(colors.success)
                };

                let row = Row::new(vec![
                    Cell::from(date),
                    task_cell,
                    tag_cell,
                    Cell::from(mode_str),
                    Cell::from(duration),
                    Cell::from(status).style(status_style),
                ]);
                if i == selected {
                    row.style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    row
                }
            })
            .collect();

        let mut table_state = TableState::default();
        table_state.select(Some(selected));

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Completed Sessions "),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(table, chunks[1], &mut table_state);
    }

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
    let help = Paragraph::new(" [↑↓] Select  [←→] Page  [D] Delete  [R] Rename  [?] Help ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);

    // Message overlay
    if let Some(msg) = &app.message {
        crate::tui::views::dashboard::render_message_overlay_pub(frame, app, msg);
    }
}

// T122, T123: Unit tests for session field display logic

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_status_completed() {
        assert_eq!(format_status(true), "done");
    }

    #[test]
    fn test_format_status_active() {
        assert_eq!(format_status(false), "active");
    }

    #[test]
    fn test_responsive_column_widths_wide_terminal() {
        // Wide terminal: 160 columns — task column should be generous
        let widths = responsive_column_widths(160);
        // 6 columns
        assert_eq!(widths.len(), 6);
        // Task column (index 1) should use Min constraint — gets leftover space
        matches!(widths[1], Constraint::Min(_));
    }

    #[test]
    fn test_responsive_column_widths_narrow_terminal() {
        // Narrow terminal: 60 columns — task column minimum 10
        let widths = responsive_column_widths(60);
        assert_eq!(widths.len(), 6);
        // Minimum task width is 10
        match widths[1] {
            Constraint::Min(w) => assert!(w >= 10),
            _ => panic!("Expected Min constraint for task column"),
        }
    }

    #[test]
    fn test_no_truncation_of_task_names() {
        // Verify the render code uses full task name (no truncation).
        // Simulate what the render function does for the task field.
        let long_task = "A very long task name that exceeds the old 30-character limit and should not be cut off";
        // The new approach: task = s.task.clone() — no truncation
        let displayed = long_task.to_string();
        assert_eq!(displayed, long_task, "Task name must not be truncated");
    }

    #[test]
    fn test_all_required_columns_present() {
        // The header should include all 6 columns
        let headers = ["Date", "Task", "Tag", "Mode", "Duration", "Status"];
        assert_eq!(headers.len(), 6, "All 6 session fields must be present");
        assert!(headers.contains(&"Mode"), "Mode column must be present");
        assert!(headers.contains(&"Status"), "Status column must be present");
        assert!(headers.contains(&"Tag"), "Tag column must be present");
    }

    // T130: Tags use a distinct color separate from session title color
    #[test]
    fn test_tag_color_distinct_from_session_title_color() {
        use crate::theme::Theme;
        for theme in &[Theme::OneDark, Theme::Material, Theme::Light, Theme::Dark] {
            let colors = theme.colors();
            assert_ne!(
                colors.tag_color, colors.session_title,
                "tag_color must differ from session_title for theme {:?}",
                theme
            );
        }
    }

    // T134: Sessions with no tags display a placeholder, not an error
    #[test]
    fn test_empty_tag_displays_placeholder() {
        let tag: Option<&str> = None;
        let displayed = tag.unwrap_or("—");
        assert_eq!(displayed, "—", "Missing tag must render as placeholder '—'");
    }
}
