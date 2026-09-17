use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, Timelike, Weekday};

/// Inclusive start (week_start 00:00:00) and exclusive end (next week_start 00:00:00).
pub fn week_bounds(now: DateTime<Local>, week_start: Weekday) -> (DateTime<Local>, DateTime<Local>) {
    let today = now.date_naive();
    let days_since_start = days_since(today.weekday(), week_start);
    let start_date = today - Duration::days(days_since_start);
    let end_date = start_date + Duration::days(7);
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("valid midnight");
    let start = start_date
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .expect("local week start");
    let end = end_date
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .expect("local week end");
    (start, end)
}

fn days_since(today: Weekday, week_start: Weekday) -> i64 {
    let today_num = today.num_days_from_monday() as i64;
    let start_num = week_start.num_days_from_monday() as i64;
    (today_num - start_num).rem_euclid(7)
}

pub fn month_bounds(now: DateTime<Local>) -> (DateTime<Local>, DateTime<Local>) {
    let date = now.date_naive();
    let start_date = date.with_day(1).expect("day 1 of month");
    let end_date = if date.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).expect("next year")
    } else {
        chrono::NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).expect("next month")
    };
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("valid midnight");
    let start = start_date
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .expect("local month start");
    let end = end_date
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .expect("local month end");
    (start, end)
}

pub fn day_bounds(now: DateTime<Local>) -> (DateTime<Local>, DateTime<Local>) {
    let date = now.date_naive();
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("valid midnight");
    let start = date
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .expect("local day start");
    let end = start + Duration::days(1);
    (start, end)
}

/// Anchor used when logging a task on a day other than today (midday avoids DST edges).
pub fn day_anchor(date: chrono::NaiveDate) -> DateTime<Local> {
    let noon = NaiveTime::from_hms_opt(12, 0, 0).expect("valid noon");
    date.and_time(noon)
        .and_local_timezone(Local)
        .single()
        .expect("local day anchor")
}

pub fn format_duration_minutes(total_minutes: i64) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

/// Fixed-width `HH:MM` (always 5 chars while hours < 100).
pub fn format_hhmm(total_minutes: i64) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours:02}:{minutes:02}")
}

pub fn format_clock(dt: &DateTime<Local>) -> String {
    format!("{:02}:{:02}", dt.hour(), dt.minute())
}

/// First visible index so `selected` stays in a window of `capacity` rows.
pub fn window_start(selected: usize, len: usize, capacity: usize) -> usize {
    if capacity == 0 || len <= capacity {
        return 0;
    }
    let max_start = len - capacity;
    if selected < capacity {
        0
    } else {
        (selected + 1 - capacity).min(max_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn week_starts_thursday_by_default() {
        // Wed 16 Sep 2026 (Thu week started 10 Sep); exclusive end is next Thu 17 Sep
        let wed = Local.with_ymd_and_hms(2026, 9, 16, 12, 0, 0).unwrap();
        let (start, end) = week_bounds(wed, Weekday::Thu);
        assert_eq!(start.weekday(), Weekday::Thu);
        assert_eq!(
            start.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap()
        );
        assert_eq!(
            end.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 17).unwrap()
        );

        let thu = Local.with_ymd_and_hms(2026, 9, 17, 8, 0, 0).unwrap();
        let (start, end) = week_bounds(thu, Weekday::Thu);
        assert_eq!(
            start.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 17).unwrap()
        );
        assert_eq!(
            end.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 24).unwrap()
        );
    }

    #[test]
    fn week_can_start_monday() {
        let wed = Local.with_ymd_and_hms(2026, 9, 16, 12, 0, 0).unwrap();
        let (start, end) = week_bounds(wed, Weekday::Mon);
        assert_eq!(start.weekday(), Weekday::Mon);
        assert_eq!(
            start.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 14).unwrap()
        );
        assert_eq!(
            end.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 9, 21).unwrap()
        );
    }

    #[test]
    fn hhmm_is_fixed_width() {
        assert_eq!(format_hhmm(0), "00:00");
        assert_eq!(format_hhmm(90), "01:30");
        assert_eq!(format_hhmm(6 * 60), "06:00");
        assert_eq!(format_hhmm(30 * 60), "30:00");
        let suffix = format!(" {} / {}", format_hhmm(0), format_hhmm(6 * 60));
        assert_eq!(suffix, " 00:00 / 06:00");
        assert_eq!(suffix.chars().count(), 14);
    }

    #[test]
    fn window_keeps_selection_visible() {
        assert_eq!(window_start(0, 20, 5), 0);
        assert_eq!(window_start(4, 20, 5), 0);
        assert_eq!(window_start(5, 20, 5), 1);
        assert_eq!(window_start(19, 20, 5), 15);
        assert_eq!(window_start(0, 3, 5), 0);
    }
}
