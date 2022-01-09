use chrono::{DateTime, NaiveDateTime, Utc};

pub fn unix_ms_to_utc(ts: i64) -> DateTime<Utc> {
    let dt = NaiveDateTime::from_timestamp(ts / 1000,
        (ts % 1000) as u32 * 1_000_000);
    DateTime::from_utc(dt, Utc)
}
