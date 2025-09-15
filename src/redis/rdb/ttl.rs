use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub enum Ttl {
    None,
    Seconds(u32),
    Milliseconds(u64),
}

impl Ttl {
    pub fn is_expired(&self) -> bool {
        match self {
            Ttl::None => false,
            Ttl::Seconds(seconds) => {
                DateTime::from_timestamp_millis(*seconds as i64 * 1000).unwrap() <= Utc::now()
            }
            Ttl::Milliseconds(milliseconds) => {
                DateTime::from_timestamp_millis(*milliseconds as i64).unwrap() <= Utc::now()
            }
        }
    }
}
