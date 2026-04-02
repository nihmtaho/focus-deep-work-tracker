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
    /// Start a new work session
    Start {
        /// Description of the work to be done
        task: String,
        /// Category label for the session
        #[arg(short, long)]
        tag: Option<String>,
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
        Commands::Start { task, tag } => commands::start::run(&conn, task, tag)?,
        Commands::Stop => commands::stop::run(&conn)?,
        Commands::Status => commands::status::run(&conn)?,
        Commands::Log { limit } => commands::log::run(&conn, limit)?,
        Commands::Report { today, week } => commands::report::run(&conn, today, week)?,
        Commands::Export { format } => commands::export::run(&conn, format)?,
    }

    Ok(())
}
