pub mod app;
pub mod events;
pub mod ui;
pub mod views;

use std::io::{self, IsTerminal};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::tui::app::{App, View};
use crate::tui::events::handle_key_event;
use crate::tui::ui::render;

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 12;

/// Entry point for the TUI. Called from main.rs for `focus ui`.
pub fn run(conn: rusqlite::Connection) -> Result<()> {
    // TTY guard
    if !io::stdout().is_terminal() {
        eprintln!("focus ui requires an interactive terminal (TTY).");
        return Err(anyhow::anyhow!("Not a TTY"));
    }

    // Read NO_COLOR
    let no_color = std::env::var("NO_COLOR").is_ok();

    // Set up panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // Enable raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &conn, no_color);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &rusqlite::Connection,
    no_color: bool,
) -> Result<()> {
    let mut app = App::new(no_color);

    // Load initial dashboard data
    app.load_dashboard(conn)?;

    loop {
        // Check terminal size
        let size = terminal.size()?;
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            // Show size guard message
            terminal.draw(|frame| {
                let area = frame.area();
                let msg = format!(
                    "Terminal too small. Minimum: {}×{}. Current: {}×{}. Resize to continue or press Q to quit.",
                    MIN_WIDTH, MIN_HEIGHT, area.width, area.height
                );
                let widget = ratatui::widgets::Paragraph::new(msg)
                    .style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Yellow),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: true });
                frame.render_widget(widget, area);
            })?;

            // Poll for resize or Q
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                        return Ok(());
                    }
                }
            }
            continue;
        }

        // Draw current state
        terminal.draw(|frame| render(frame, &app))?;

        // Poll for events with 100ms timeout (for timer ticks)
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    let should_quit = handle_key_event(&mut app, conn, key)?;
                    if should_quit || app.quit_pending {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {
                    // Will re-check size next loop iteration
                }
                _ => {}
            }
        } else {
            // Tick: update dashboard timer if in Dashboard view
            if matches!(app.view, View::Dashboard) {
                app.tick_dashboard(conn)?;
            }
        }
    }
}
