use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::utils::logging::LogEvent;

pub static EVENT_BUFFER_PRIMARY: Lazy<Mutex<Vec<LogEvent>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static EVENT_BUFFER_SECONDARY: Lazy<Mutex<Vec<LogEvent>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub async fn push_to_batch(event: LogEvent) {
    if let Ok(mut primary) = EVENT_BUFFER_PRIMARY.try_lock() {
        primary.push(event);
    } else {
        let mut secondary = EVENT_BUFFER_SECONDARY.lock().await;
        secondary.push(event);
    }
}
