use crate::{utils::logging::log_event, values::config::get_config};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub static EVENT_CHANNEL: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(100);
    tx
});

pub async fn push_event(event: String) {
    let _ = EVENT_CHANNEL.send(event.clone());
    if get_config().app.event_logging {
        log_event(event).await;
    }
}
