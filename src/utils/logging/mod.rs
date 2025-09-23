use crate::values::config::get_config;
use chrono::Utc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(serde::Serialize)]
struct LogEvent<'a> {
    timestamp: String,
    level: &'a str,
    target: &'a str,
    message: &'a str,
    #[serde(rename = "type")]
    log_type: &'a str,
}

pub async fn log_event(message: String) {
    let event = LogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "INFO",
        target: "rodan.events",
        message: &message,
        log_type: " notifications",
    };
    if let Ok(json_event) = serde_json::to_string(&event) {
        if let Some(path) = &get_config().app.events_logfile {
            let line = format!("{}\n", json_event);
            match fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(line.as_bytes()).await {
                        eprintln!("Failed to write log event: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to open log file: {}", e),
            }
        }
    }
}
// TODO: batch this process .. and then drain it when shutting down
