pub mod app;
pub mod events;
pub mod ui;
pub mod views;

use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::config::{config_file_path, load_config};
use crate::db::session_store;
use crate::tui::app::{App, Tab};
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

    // Load config
    let config = load_config(&config_file_path());

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

    let result = run_app(&mut terminal, &conn, no_color, config);

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
    config: crate::config::AppConfig,
) -> Result<()> {
    // Signal flag for Ctrl+C / SIGTERM
    let signal_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&signal_flag);
    let _ = ctrlc::set_handler(move || {
        flag_clone.store(true, Ordering::Relaxed);
    });

    let mut app = App::new(no_color, config);

    // Load initial dashboard data
    app.load_dashboard(conn)?;

    loop {
        // Check signal
        if signal_flag.load(Ordering::Relaxed) {
            break;
        }

        // Check terminal size
        let size = terminal.size()?;
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            terminal.draw(|frame| {
                let area = frame.area();
                let msg = format!(
                    "Terminal too small. Minimum: {}×{}. Current: {}×{}. Resize or press Q to quit.",
                    MIN_WIDTH, MIN_HEIGHT, area.width, area.height
                );
                let widget = ratatui::widgets::Paragraph::new(msg)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                frame.render_widget(widget, area);
            })?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                        break;
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
                        break;
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        } else {
            // Tick: update dashboard data or pomodoro timer
            if app.active_tab == Tab::Pomodoro {
                if let Some(ref mut timer) = app.pomodoro_timer {
                    let events = timer.tick_secs(1, conn)?;
                    for event in events {
                        use crate::pomodoro::timer::TimerEvent;
                        match event {
                            TimerEvent::PhaseComplete { to, .. } => {
                                use crate::models::pomodoro::PomodoroPhase;
                                let (title, body) = match to {
                                    PomodoroPhase::Work => ("Focus!", "Break over — time to work."),
                                    PomodoroPhase::Break => ("Break time!", "Work phase complete."),
                                    PomodoroPhase::LongBreak => {
                                        ("Long break!", "Take a longer rest.")
                                    }
                                };
                                app.message = Some(crate::tui::app::MessageOverlay::success(
                                    body.to_string(),
                                ));
                                crate::pomodoro::notify::send_notification(title, body);
                            }
                            TimerEvent::AutoAbandoned { .. } => {
                                app.pomodoro_timer = None;
                                app.active_tab = Tab::Dashboard;
                                let _ = app.load_dashboard(conn);
                                app.message = Some(crate::tui::app::MessageOverlay::error(
                                    "Pomodoro abandoned: paused too long.",
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            } else if app.active_tab == Tab::Dashboard {
                app.tick_dashboard(conn)?;
            }
        }
    }

    // Auto-save active session on any exit path
    if let Ok(Some(_)) = session_store::get_active_session(conn) {
        let _ = session_store::stop_session(conn);
    }

    Ok(())
}
