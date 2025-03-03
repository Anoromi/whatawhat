use std::sync::atomic::AtomicU64;

use chrono::{DateTime, Duration, NaiveDate, Utc};

static TIME_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn get_current_time() -> DateTime<Utc> {
    let offset = TIME_OFFSET.load(std::sync::atomic::Ordering::SeqCst);

    let time = Utc::now();
    time + Duration::nanoseconds(offset.try_into().unwrap())
}

pub fn advance_time(duration: Duration) {
    let nanoseconds = duration
        .num_nanoseconds()
        .expect("Duration should provide proper nanoseconds");
    debug_assert!(nanoseconds > 0, "Can't go backwards in time");
    TIME_OFFSET.fetch_add(
        nanoseconds.try_into().unwrap(),
        std::sync::atomic::Ordering::SeqCst,
    );
}


pub fn date_to_record_name(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}
