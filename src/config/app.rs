use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Default, Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "auth-required")]
    pub auth_required: bool,
    pub events: Option<EventsConfig>,
    #[serde(rename = "event-logging")]
    pub event_logging: bool,
    #[serde(rename = "event-log-file")]
    pub events_logfile: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
pub struct EventsConfig {
    pub kafka: Option<KafkaConfig>,
    pub http: Option<HttpConfig>,
}

#[derive(Default, Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub group_id: String,
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
        if self.event_logging && !self.events_logfile.is_some() {
            return Err("app: event-logging is enabled but no log file is given".into());
        }
        Ok(())
    }
}

impl EventsConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut count = 0;
        if self.kafka.is_some() {
            count += 1;
        }
        if self.http.is_some() {
            count += 1;
        }
        if count == 0 {
            return Err(
                "events: at least one event type (kafka or http) must be configured".into(),
            );
        }
        if count > 1 {
            return Err(
                "events: only one event type can be configured at a time (choose kafka OR http)"
                    .into(),
            );
        }
        if let Some(kafka) = &self.kafka {
            kafka.validate()?;
        }
        if let Some(http) = &self.http {
            http.validate()?;
        }
        Ok(())
    }
}

impl KafkaConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.brokers.is_empty() {
            return Err("events.kafka.brokers cannot be empty".into());
        }
        if self.topic.trim().is_empty() {
            return Err("events.kafka.topic cannot be empty".into());
        }
        if self.group_id.trim().is_empty() {
            return Err("events.kafka.group_id cannot be empty".into());
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
