use actix_web::{App, HttpServer};
use env_logger::Env;
use rodan_sse::{config, router::create_app, utils, values};

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
    if cfg.app.event_logging && cfg.app.events_logfile.is_some() {
        let flush_duration = cfg
            .app
            .event_flush_duration
            .unwrap_or_else(|| std::time::Duration::from_secs(300));
        let rotate_duration = cfg.app.event_log_rotation.unwrap_or(flush_duration * 12);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(flush_duration.min(rotate_duration)).await;
                utils::flush_events().await;
                utils::rotate_logs().await;
            }
        });
    }
    values::config::set_config(cfg);
    let server = HttpServer::new(|| App::new().configure(create_app))
        .bind(addr)?
        .run();
    tokio::select! {
        res = server => res,
        _ = tokio::signal::ctrl_c() => {
            println!("Flushing logs....!");
            utils::flush_events().await;
            Ok(())
        }
    }
}
