mod handlers;

use crate::{
    responses::{not_found_handler, ping_response},
    utils::middlewares,
    values,
};
use actix_web::{
    middleware::{Logger, from_fn},
    web,
};

pub fn create_app(cfg: &mut web::ServiceConfig) {
    let config = values::config::get_config();
    let mut api_scope = web::scope("/api")
        .route("/ping", web::get().to(ping_response))
        .route("/notify", web::get().to(handlers::sse_handler))
        .route("/events", web::get().to(handlers::events_get_handler));
    if let Some(events) = &config.app.events {
        if let Some(http) = &events.http {
            api_scope = api_scope.route(&http.endpoint, web::post().to(handlers::events_ingestor))
        }
    } else {
        panic!("No events ingestion path is specified")
    }
    if config.app.auth_required {
        if config.server.production {
            cfg.service(
                api_scope
                    .wrap(from_fn(middlewares::auth::auth_middleware))
                    .wrap(from_fn(middlewares::log::log_middleware))
                    .wrap(middlewares::cors::cors_middleware()),
            );
        } else {
            cfg.service(
                api_scope
                    .wrap(from_fn(middlewares::auth::auth_middleware))
                    .wrap(Logger::default())
                    .wrap(middlewares::cors::cors_middleware()),
            );
        }
    } else {
        if config.server.production {
            cfg.service(
                api_scope
                    .wrap(from_fn(middlewares::log::log_middleware))
                    .wrap(middlewares::cors::cors_middleware()),
            );
        } else {
            cfg.service(
                api_scope
                    .wrap(Logger::default())
                    .wrap(middlewares::cors::cors_middleware()),
            );
        }
    }
    cfg.default_service(web::route().to(not_found_handler));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::config::set_config;
    use crate::{
        config::{
            Config,
            app::{AppConfig, EventsConfig, HttpConfig},
            server::ServerConfig,
        },
        values::config::get_config,
    };
    use actix_web::http::header::CONTENT_TYPE;
    use actix_web::{App, http::StatusCode, test};

    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_app_routes() {
        crate::utils::events::flush_events().await;
        let cfg = Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 8080,
                production: false,
                cors_url: vec!["http://localhost:3000".into()],
                security: Default::default(),
            },
            app: AppConfig {
                auth_required: false,
                events: Some(EventsConfig {
                    http: Some(HttpConfig {
                        endpoint: "/ingest/event".into(),
                        api_key: Some("1234567890123456".into()),
                        hashed_api_key: Some("1234567890123456".into()),
                    }),
                }),
                event_logging: false,
                event_log_rotation: None,
                events_logfile: Some("events.log".into()),
                event_max_segments: Some(10),
                event_segment_size: Some(100),
            },
        };
        set_config(cfg);
        let app = test::init_service(App::new().configure(create_app)).await;
        let req = test::TestRequest::get().uri("/api/ping").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let req = test::TestRequest::get().uri("/api/events").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let req = test::TestRequest::post()
            .uri("/api/ingest/event")
            .insert_header((
                "x-api-key",
                get_config()
                    .app
                    .events
                    .as_ref()
                    .unwrap()
                    .http
                    .as_ref()
                    .unwrap()
                    .hashed_api_key
                    .as_ref()
                    .unwrap()
                    .to_string(),
            ))
            .insert_header((CONTENT_TYPE, "application/json"))
            .set_payload(r#"{"events": ["test"]}"#)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status().is_success(),
            "Expected success for /api/ingest/event, got {:?}",
            resp.status()
        );
        let events = crate::utils::events::get_events(None).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].payload, "test");
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_app_no_events() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let cfg = Config {
            server: ServerConfig::default(),
            app: AppConfig {
                auth_required: false,
                events: None,
                event_logging: false,
                event_log_rotation: None,
                events_logfile: None,
                event_max_segments: Some(10),
                event_segment_size: Some(100),
            },
        };
        set_config(cfg);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = actix_web::test::init_service(actix_web::App::new().configure(create_app));
        }));
        assert!(result.is_err(), "Expected panic when events config is None");
    }
}
