mod batch;
mod flush;
mod rotate;

pub use flush::flush_events;
pub use rotate::rotate_logs;

use crate::utils::logging::batch::push_to_batch;
use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize)]
struct LogEvent {
    timestamp: String,
    level: String,
    target: String,
    message: String,
    #[serde(rename = "type")]
    log_type: String,
}

pub async fn log_event(message: String) {
    let event = LogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "INFO".into(),
        target: "rodan.events".into(),
        message: message,
        log_type: "notifications".into(),
    };
    push_to_batch(event).await;
}
