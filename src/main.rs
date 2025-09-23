use actix_web::{App, HttpServer};
use env_logger::Env;
use rodan_sse::{config, router::create_app, values};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let cfg_file = std::env::var("CONFIG_FILE").expect("Failed to read CONFIG_FILE env var");
    let cfg = config::Config::from_file(&cfg_file)
        .await
        .expect("Failed to load config");
    let host: String = cfg.server.host.clone();
    let port: u32 = cfg.server.port;
    let addr: String = format!("{}:{}", host, port);
    values::config::set_config(cfg);
    HttpServer::new(|| App::new().configure(create_app))
        .bind(addr)?
        .run()
        .await
}
