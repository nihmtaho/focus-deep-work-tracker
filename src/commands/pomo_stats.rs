use anyhow::Result;
use rusqlite::Connection;

use crate::db::pomodoro_store;
use crate::display::format::format_duration_mins;

pub fn run(conn: &Connection, today: bool, week: bool) -> Result<()> {
    if week {
        run_week(conn)
    } else {
        // Default: today
        let _ = today;
        run_today(conn)
    }
}

fn run_today(conn: &Connection) -> Result<()> {
    let date = pomodoro_store::today_local_date();
    let stats = pomodoro_store::get_stats_for_date(conn, &date)?;

    if stats.completed == 0 && stats.abandoned == 0 {
        println!("No Pomodoro sessions today.");
        return Ok(());
    }

    // Calculate streak from last 30 days
    let (from_date, _) = date_range(30);
    let history = pomodoro_store::get_stats_range(conn, &from_date, &date)?;
    let streak = pomodoro_store::calculate_streak(&history);

    let local_now = chrono::Local::now();
    let display_date = local_now.format("%A, %-d %B %Y").to_string();

    println!("Pomodoro Statistics — {display_date}");
    println!("{}", "─".repeat(42));
    println!("{:<22}: {}", "Completed pomodoros", stats.completed);
    println!("{:<22}: {}", "Abandoned", stats.abandoned);
    println!(
        "{:<22}: {}",
        "Work time",
        format_duration_mins(stats.work_minutes)
    );
    println!(
        "{:<22}: {}",
        "Break time",
        format_duration_mins(stats.break_minutes)
    );
    println!("{:<22}: {} days", "Current streak", streak);

    Ok(())
}

fn run_week(conn: &Connection) -> Result<()> {
    let today = pomodoro_store::today_local_date();
    let (from_date, dates) = date_range(7);

    let all_stats = pomodoro_store::get_stats_range(conn, &from_date, &today)?;
    let best_streak = pomodoro_store::calculate_best_streak(&all_stats);

    // Build a lookup map by date
    let map: std::collections::HashMap<String, _> =
        all_stats.into_iter().map(|s| (s.date.clone(), s)).collect();

    if map.values().all(|s| s.completed == 0 && s.abandoned == 0)
        && map.len() < dates.len()
        && map.is_empty()
    {
        println!("No Pomodoro sessions in the past 7 days.");
        return Ok(());
    }

    let all_empty = dates.iter().all(|d| {
        map.get(d)
            .is_none_or(|s| s.completed == 0 && s.abandoned == 0)
    });
    if all_empty {
        println!("No Pomodoro sessions in the past 7 days.");
        return Ok(());
    }

    println!("Pomodoro Statistics — Last 7 Days");
    let separator = "─".repeat(64);
    println!("{separator}");
    println!(
        "{:<12} {:>10} {:>10} {:>12} {:>12}",
        "Date", "Completed", "Abandoned", "Work time", "Break time"
    );
    println!("{separator}");

    let mut total_completed = 0u32;
    let mut total_abandoned = 0u32;
    let mut total_work = 0u32;
    let mut total_break = 0u32;

    for date in &dates {
        let empty = crate::models::pomodoro::PomodoroStats {
            date: date.clone(),
            ..Default::default()
        };
        let s = map.get(date).unwrap_or(&empty);
        println!(
            "{:<12} {:>10} {:>10} {:>12} {:>12}",
            s.date,
            s.completed,
            s.abandoned,
            format_duration_mins(s.work_minutes),
            format_duration_mins(s.break_minutes),
        );
        total_completed += s.completed;
        total_abandoned += s.abandoned;
        total_work += s.work_minutes;
        total_break += s.break_minutes;
    }

    println!("{separator}");
    println!(
        "{:<12} {:>10} {:>10} {:>12} {:>12}",
        "Total",
        total_completed,
        total_abandoned,
        format_duration_mins(total_work),
        format_duration_mins(total_break),
    );
    println!("Best streak : {} days", best_streak);

    Ok(())
}

/// Returns (from_date_str, list_of_date_strings_newest_first) for the past `days` days.
fn date_range(days: i64) -> (String, Vec<String>) {
    let today = chrono::Local::now().date_naive();
    let mut dates = Vec::new();
    for i in (0..days).rev() {
        let d = today - chrono::Duration::days(i);
        dates.push(d.format("%Y-%m-%d").to_string());
    }
    let from = dates.first().cloned().unwrap_or_default();
    (from, dates)
}
