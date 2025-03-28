
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone};


/// This is the standard way of converting a date to a string in whatawhat.
pub fn date_to_record_name(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Returns start of the next day.
pub fn next_day_start<Tz: TimeZone>(date: DateTime<Tz>) -> DateTime<Tz> {
    (date + Duration::days(1)).with_time(NaiveTime::MIN).unwrap()
}
