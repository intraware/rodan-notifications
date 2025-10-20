use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub static EVENT_CHANNEL: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(100);
    tx
});
