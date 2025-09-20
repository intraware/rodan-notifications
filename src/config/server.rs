use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u32,
    pub production: bool,
    pub security: self::SecurityConfig,
}

#[derive(Default, Debug, Deserialize)]
pub struct SecurityConfig {
    #[serde(rename = "jwt-secret")]
    pub jwt_secret: String,
}
