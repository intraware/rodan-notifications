mod handlers;

use super::router::handlers::{ingester, sse};
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
        .route("/notify", web::get().to(sse::sse_handler));
    if let Some(events) = &config.app.events {
        if let Some(http) = &events.http {
            api_scope = api_scope.route(&http.endpoint, web::post().to(ingester::events_ingestor))
        }
    }
    if config.app.auth_required {
        if config.server.production {
            cfg.service(
                api_scope
                    .wrap(from_fn(middlewares::auth::auth_middleware))
                    .wrap(from_fn(middlewares::log::log_middleware)),
            );
        } else {
            cfg.service(
                api_scope
                    .wrap(from_fn(middlewares::auth::auth_middleware))
                    .wrap(Logger::default()),
            );
        }
    } else {
        if config.server.production {
            cfg.service(api_scope.wrap(from_fn(middlewares::log::log_middleware)));
        } else {
            cfg.service(api_scope.wrap(Logger::default()));
        }
    }
    cfg.default_service(web::route().to(not_found_handler));
}
