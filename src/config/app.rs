use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "auth-required")]
    pub auth_required: bool,
    pub events: Option<EventsConfig>,
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

#[derive(Default, Debug, Deserialize)]
pub struct HttpConfig {
    pub endpoint: String,
    #[serde(rename = "api-key")]
    pub api_key: Option<String>,
}
