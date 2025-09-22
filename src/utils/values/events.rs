use crate::values::config::get_config;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub static EVENT_CHANNEL: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(100);
    tx
});

pub fn push_event(event: String) {
    let _ = EVENT_CHANNEL.send(event);
    if get_config().app.event_logging {
        // log events
    }
}
