use chrono::Utc;

pub type DateProvider = Box<dyn FnMut() -> chrono::DateTime<Utc> + 'static>;
