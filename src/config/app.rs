use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Default, Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "auth-required")]
    pub auth_required: bool,
    pub events: Option<EventsConfig>,
    #[serde(rename = "event-logging")]
    pub event_logging: bool,
    #[serde(rename = "event-log-file")]
    pub events_logfile: Option<String>,
    #[serde(rename = "event-log-rotation")]
    #[serde(with = "humantime_serde")]
    pub event_log_rotation: Option<Duration>,
    #[serde(rename = "event-segment-size")]
    pub event_segment_size: Option<usize>,
    #[serde(rename = "event-max-segments")]
    pub event_max_segments: Option<usize>,
}

#[derive(Default, Debug, Deserialize)]
pub struct EventsConfig {
    pub http: Option<HttpConfig>,
}

#[derive(Default, Debug)]
pub struct HttpConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub hashed_api_key: Option<String>,
}

impl<'de> Deserialize<'de> for HttpConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawHttpConfig {
            endpoint: String,
            #[serde(rename = "api-key")]
            api_key: Option<String>,
        }
        let raw = RawHttpConfig::deserialize(deserializer)?;
        let hashed_api_key = raw.api_key.as_ref().map(|key| {
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            format!("{:x}", hasher.finalize())
        });
        Ok(HttpConfig {
            endpoint: raw.endpoint,
            api_key: raw.api_key,
            hashed_api_key,
        })
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(events) = &self.events {
            events.validate()?;
        }
        if self.event_logging && self.events_logfile.is_none() {
            return Err("app: event-logging is enabled but no log file is given".into());
        }
        if let Some(size) = self.event_segment_size {
            if size == 0 {
                return Err("app: event-segment-size must be greater than 0".into());
            }
        }
        if let Some(max) = self.event_max_segments {
            if max == 0 {
                return Err("app: event-max-segments must be greater than 0".into());
            }
        }
        Ok(())
    }
}

impl EventsConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut count = 0;
        if self.http.is_some() {
            count += 1;
        }
        if count == 0 {
            return Err("events: at least one event type (http) must be configured".into());
        }
        if count > 1 {
            return Err(
                "events: only one event type can be configured at a time (choose http)".into(),
            );
        }
        if let Some(http) = &self.http {
            http.validate()?;
        }
        Ok(())
    }
}

impl HttpConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.endpoint.trim().is_empty() {
            return Err("events.http.endpoint cannot be empty".into());
        }
        if self.api_key.is_some() && self.hashed_api_key.is_none() {
            return Err("events.http.api-key was provided but hashing failed".into());
        }
        if let Some(api_key) = &self.api_key {
            if api_key.len() < 16 {
                return Err("events.http.api-key must be at least 16 characters".into());
            }
        }
        Ok(())
    }
}
