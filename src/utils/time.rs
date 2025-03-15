
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone};

pub fn date_to_record_name(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn next_day_start<Tz: TimeZone>(date: DateTime<Tz>) -> DateTime<Tz> {
    (date + Duration::days(1)).with_time(NaiveTime::MIN).unwrap()
}
