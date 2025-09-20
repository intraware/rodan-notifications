pub mod app;
pub mod server;

#[derive(Default, Debug, serde::Deserialize)]
pub struct Config {
    pub server: server::ServerConfig,
    pub app: app::AppConfig,
}

impl Config {
    pub async fn from_file(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let contents = tokio::fs::read_to_string(file_path).await?;
        let cfg: Config = toml::from_str(&contents)?;
        Ok(cfg)
    }
}
