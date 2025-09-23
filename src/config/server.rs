use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u32,
    pub production: bool,
    #[serde(rename = "cors-url")]
    pub cors_url: Vec<String>,
    pub security: self::SecurityConfig,
}

#[derive(Default, Debug, Deserialize)]
pub struct SecurityConfig {
    #[serde(rename = "jwt-secret")]
    pub jwt_secret: String,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("server.host cannot be empty".into());
        }
        if self.port == 0 {
            return Err("server.port must be greater than 0".into());
        }
        if self.cors_url.is_empty() {
            return Err("server.cors-url must contain at least one entry".into());
        }
        self.security.validate()?;
        Ok(())
    }
}

impl SecurityConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.trim().is_empty() {
            return Err("server.security.jwt-secret cannot be empty".into());
        }
        if self.jwt_secret.len() < 8 {
            return Err("server.security.jwt-secret must be at least 8 characters".into());
        }
        Ok(())
    }
}
