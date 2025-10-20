use chrono::{DateTime, Utc};

use crate::utils::events::Event;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(serde::Serialize)]
pub struct PingResponse {
    pub msg: String,
}

#[derive(serde::Serialize)]
pub struct EventResponse {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl From<Event> for EventResponse {
    fn from(event: Event) -> Self {
        EventResponse {
            timestamp: event.timestamp,
            message: event.payload,
        }
    }
}
