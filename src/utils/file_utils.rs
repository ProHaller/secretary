use chrono::{DateTime, TimeZone, Utc};
use std::time::SystemTime;

pub fn parse_timestamp_to_system_time(timestamp: &str) -> Result<SystemTime, chrono::ParseError> {
    let parsed_date = DateTime::parse_from_rfc3339(timestamp)?;
    Ok(parsed_date.with_timezone(&Utc).into())
}
