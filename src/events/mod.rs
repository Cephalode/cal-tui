/// Events module for managing calendar events
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub description: Option<String>,
}

impl Event {
    pub fn new(title: String, start_time: NaiveDateTime, end_time: NaiveDateTime) -> Self {
        Self {
            id: uuid::generate(),
            title,
            start_time,
            end_time,
            description: None,
        }
    }
}

mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn generate() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:x}", timestamp)
    }
}
