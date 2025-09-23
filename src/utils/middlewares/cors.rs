use crate::values::config::get_config;
use actix_cors::Cors;

pub fn cors_middleware() -> Cors {
    let config = &get_config().server;
    let mut cors = Cors::default().allow_any_header().allow_any_method();
    if config.cors_url.len() == 1 && config.cors_url[0] == "*" && !config.production {
        cors = cors.allow_any_origin();
    } else {
        for origin in &config.cors_url {
            cors = cors.allowed_origin(origin);
        }
    }
    cors
}
