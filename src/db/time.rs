use chrono::{Datelike, Duration, Local, TimeZone};

pub fn today_start() -> i64 {
    let now = Local::now();
    let today = now.date_naive().and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&today)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn current_week_start() -> i64 {
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday() as i64;
    let monday = now.date_naive() - Duration::days(days_since_monday);
    let midnight = monday.and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&midnight)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn rolling_7d_start() -> i64 {
    (chrono::Utc::now() - Duration::seconds(7 * 86400)).timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn today_start_is_before_now() {
        let ts = today_start();
        assert!(ts > 0);
        assert!(ts <= chrono::Utc::now().timestamp());
    }

    #[test]
    fn today_start_is_midnight_local() {
        let ts = today_start();
        let dt = chrono::Local.timestamp_opt(ts, 0).single().unwrap();
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn current_week_start_is_monday() {
        let ts = current_week_start();
        let dt = chrono::Local.timestamp_opt(ts, 0).single().unwrap();
        assert_eq!(dt.weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn rolling_7d_start_is_7_days_ago() {
        let ts = rolling_7d_start();
        let now = chrono::Utc::now().timestamp();
        let diff = now - ts;
        assert!(diff >= 604795 && diff <= 604805, "diff={diff}");
    }
}
