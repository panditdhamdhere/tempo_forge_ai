use chrono::{DateTime, Utc};

pub type UtcDateTime = DateTime<Utc>;

pub fn now() -> UtcDateTime {
    Utc::now()
}
