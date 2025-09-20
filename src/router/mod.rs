#[path = "handlers/sse.rs"]
mod sse;

use crate::{responses::ping_response, utils::middlewares, values};
use actix_web::{middleware::from_fn, web};

pub fn create_app(cfg: &mut web::ServiceConfig) {
    let config = values::config::get_config();
    if config.app.auth_required {
        cfg.service(
            web::scope("/api")
                .wrap(from_fn(middlewares::auth::auth_middleware))
                .route("/ping", web::get().to(ping_response))
                .route("/notify", web::get().to(sse::sse_handler)),
        );
    } else {
        cfg.service(
            web::scope("/api")
                .route("/ping", web::get().to(ping_response))
                .route("/notify", web::get().to(sse::sse_handler)),
        );
    }
}
