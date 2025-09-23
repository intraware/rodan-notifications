use crate::values::config::get_config;
use chrono::Utc;
use tokio::fs;

pub async fn rotate_logs() {
    let config = get_config();
    let path = match &config.app.events_logfile {
        Some(p) => p,
        None => return,
    };
    let rotation_duration = match config.app.event_log_rotation {
        Some(d) => d,
        None => return,
    };
    let metadata = match fs::metadata(&path).await {
        Ok(m) => m,
        Err(_) => return,
    };
    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return,
    };
    let age = Utc::now().signed_duration_since(chrono::DateTime::<Utc>::from(modified));
    if age.to_std().unwrap_or_default() < rotation_duration {
        return;
    }
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let rotated_file = format!("{}.{}", path, timestamp);
    if let Err(e) = fs::rename(&path, &rotated_file).await {
        eprintln!("Failed to rotate log file: {}", e);
        return;
    }
    if let Err(e) = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .await
    {
        eprintln!("Failed to create new log file after rotation: {}", e);
    }
}
