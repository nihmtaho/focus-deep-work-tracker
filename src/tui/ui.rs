use ratatui::Frame;

use crate::tui::app::{App, View};
use crate::tui::views;

/// Dispatch rendering to the appropriate view renderer.
pub fn render(frame: &mut Frame, app: &App) {
    match &app.view {
        View::Dashboard => views::dashboard::render(frame, app),
        View::Menu { selected } => views::menu::render(frame, app, *selected),
        View::StartForm {
            task,
            tag,
            active_field,
        } => views::start_form::render(frame, app, task, tag, active_field),
        View::Log { page } => views::log::render(frame, app, *page),
        View::Report {
            window,
            selected_window,
        } => views::report::render(frame, app, window, *selected_window),
    }
}
