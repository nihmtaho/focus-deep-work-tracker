use focus::commands;
use focus::db;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "focus", about = "Deep work session tracker", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new work session (add --pomodoro for Pomodoro mode)
    Start {
        /// Description of the work to be done
        task: String,
        /// Category label for the session
        #[arg(short, long)]
        tag: Option<String>,
        /// Enable Pomodoro timer mode
        #[arg(long)]
        pomodoro: bool,
        /// Work phase duration in minutes (1–120, Pomodoro mode only)
        #[arg(long)]
        work: Option<u32>,
        /// Break phase duration in minutes (1–60, Pomodoro mode only)
        #[arg(long, name = "break")]
        break_mins: Option<u32>,
        /// Long break duration in minutes (1–60, Pomodoro mode only)
        #[arg(long)]
        long_break: Option<u32>,
        /// Number of work phases before a long break (Pomodoro mode only)
        #[arg(long)]
        long_break_after: Option<u32>,
    },
    /// Stop the current active session
    Stop,
    /// Show the current session status
    Status,
    /// List completed sessions
    Log {
        /// Maximum number of sessions to show
        #[arg(short = 'n', long, default_value = "10")]
        limit: u32,
    },
    /// Show time aggregated by tag
    Report {
        /// Show today's sessions only
        #[arg(long, conflicts_with = "week")]
        today: bool,
        /// Show last 7 rolling days
        #[arg(long, conflicts_with = "today")]
        week: bool,
    },
    /// Export all session history to stdout
    Export {
        /// Output format: json or markdown
        #[arg(short, long)]
        format: String,
    },
    /// Show Pomodoro statistics
    PomoStats {
        /// Show today's statistics (default)
        #[arg(long, conflicts_with = "week")]
        today: bool,
        /// Show past 7 days as a daily breakdown
        #[arg(long, conflicts_with = "today")]
        week: bool,
    },
    /// Get or set persistent configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Launch interactive TUI dashboard
    Ui,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get the value of a config key
    Get {
        /// Config key (theme, vim-mode)
        key: String,
    },
    /// Set a config key to a value
    Set {
        /// Config key (theme, vim-mode)
        key: String,
        /// New value
        value: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = run(cli);
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let conn = db::open_db()?;

    match cli.command {
        Commands::Start {
            task,
            tag,
            pomodoro,
            work,
            break_mins,
            long_break,
            long_break_after,
        } => {
            if pomodoro {
                commands::start::run_pomodoro(
                    &conn,
                    task,
                    tag,
                    work,
                    break_mins,
                    long_break,
                    long_break_after,
                )?;
            } else {
                commands::start::run(&conn, task, tag)?;
            }
        }
        Commands::Stop => commands::stop::run(&conn)?,
        Commands::Status => commands::status::run(&conn)?,
        Commands::Log { limit } => commands::log::run(&conn, limit)?,
        Commands::Report { today, week } => commands::report::run(&conn, today, week)?,
        Commands::Export { format } => commands::export::run(&conn, format)?,
        Commands::PomoStats { today, week } => commands::pomo_stats::run(&conn, today, week)?,
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => commands::config::run_get(&key)?,
            ConfigAction::Set { key, value } => commands::config::run_set(&key, &value)?,
        },
        Commands::Ui => focus::tui::run(conn)?,
    }

    Ok(())
}
