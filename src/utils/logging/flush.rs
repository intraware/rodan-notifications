use crate::{utils::logging::batch, values::config::get_config};
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

pub async fn flush_events() {
    let path = match get_config().app.events_logfile.clone() {
        Some(p) => p,
        None => return,
    };
    let mut events_to_write = String::new();
    if let Ok(mut primary) = batch::EVENT_BUFFER_PRIMARY.try_lock() {
        if !primary.is_empty() {
            events_to_write.push_str(
                &primary
                    .drain(..)
                    .map(|e| serde_json::to_string(&e).unwrap_or_else(|_| "{}".to_string()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            events_to_write.push('\n');
        }
    }
    let mut secondary = batch::EVENT_BUFFER_SECONDARY.lock().await;
    if !secondary.is_empty() {
        events_to_write.push_str(
            &secondary
                .drain(..)
                .map(|e| serde_json::to_string(&e).unwrap_or_else(|_| "{}".to_string()))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        events_to_write.push('\n');
    }
    if events_to_write.is_empty() {
        return;
    }
    if let Some(parent) = Path::new(&path).parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            eprintln!("Failed to create log file parent directories: {}", e);
            return;
        }
    }
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(events_to_write.as_bytes()).await {
                eprintln!("Failed to write events to log file: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to open log file: {}", e),
    }
}
