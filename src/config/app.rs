#[derive(Default, Debug, serde::Deserialize)]
pub struct AppConfig {
    #[serde(rename = "auth-required")]
    pub auth_required: bool,
}
