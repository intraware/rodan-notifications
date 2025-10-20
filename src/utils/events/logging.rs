// Copyright (c) 2025 Rodan
// Licensed under the MIT License
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://opensource.org/licenses/MIT
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::Utc;
use once_cell::sync::Lazy;
use std::{path::Path, sync::Arc};
use tokio::io::AsyncWriteExt;
use tokio::{
    fs::{self, OpenOptions},
    sync::RwLock,
};

#[derive(Debug, Clone, serde::Serialize)]
struct LogEvent {
    timestamp: String,
    level: String,
    target: String,
    message: String,
    #[serde(rename = "type")]
    log_type: String,
}

fn log_event(message: String) -> LogEvent {
    LogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "INFO".into(),
        target: "rodan.events".into(),
        message: message,
        log_type: "notifications".into(),
    }
}

pub struct Log {
    events: RwLock<Vec<LogEvent>>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_event(&self, message: String) {
        let mut events = self.events.write().await;
        events.push(log_event(message));
    }

    pub async fn write_events(&self, path: String) {
        let mut events = self.events.write().await;
        if events.is_empty() {
            return;
        }
        let serialized = events
            .iter()
            .map(|e| serde_json::to_string(e).unwrap_or_else(|_| "{}".into()))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        events.clear();
        drop(events);
        if let Some(parent) = Path::new(&path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                eprintln!("Failed to create log directory: {}", e);
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
                if let Err(e) = file.write_all(serialized.as_bytes()).await {
                    eprintln!("Failed to write events to log file: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to open log file: {}", e),
        }
    }
}

pub static GLOBAL_LOG: Lazy<Arc<Log>> = Lazy::new(|| Arc::new(Log::new()));

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    async fn test_add_event() {
        let log = Log::new();

        log.add_event("Test message 1".into()).await;
        log.add_event("Test message 2".into()).await;

        let events = log.events.read().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].message, "Test message 1");
        assert_eq!(events[1].message, "Test message 2");
    }

    #[tokio::test]
    #[serial]
    async fn test_global_log() {
        let mut events = GLOBAL_LOG.events.write().await;
        events.clear();
        drop(events);
        GLOBAL_LOG.add_event("Global event 1".into()).await;
        GLOBAL_LOG.add_event("Global event 2".into()).await;
        let events = GLOBAL_LOG.events.read().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].message, "Global event 1");
        assert_eq!(events[1].message, "Global event 2");
    }

    #[tokio::test]
    async fn test_flush_events_clears_log() {
        let log = Log::new();
        log.add_event("Flush test 1".into()).await;
        log.add_event("Flush test 2".into()).await;
        log.write_events("events.log".into()).await;
        let events_after = log.events.read().await;
        assert!(events_after.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_global_flush_events_clears_log() {
        let mut events = GLOBAL_LOG.events.write().await;
        events.clear();
        drop(events);
        GLOBAL_LOG.add_event("Global Flush test 1".into()).await;
        GLOBAL_LOG.add_event("Global Flush test 2".into()).await;
        GLOBAL_LOG.write_events("events.log".into()).await;
        let events_after = GLOBAL_LOG.events.read().await;
        assert!(events_after.is_empty());
    }
}
