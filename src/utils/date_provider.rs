use chrono::Utc;

pub type DateTimeProvider<Tz = Utc> = Box<dyn FnMut() -> chrono::DateTime<Tz> + 'static>;
