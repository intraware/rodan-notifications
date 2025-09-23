use crate::{
    responses::types,
    values::{config::get_config, events::push_event},
};
use actix_web::{HttpResponse, Responder, web};
use sha2::{Digest, Sha256};

#[derive(serde::Deserialize)]
pub struct EventsPayload {
    pub events: Vec<String>,
}

pub async fn events_ingestor(
    payload: web::Json<EventsPayload>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let cfg = get_config();
    let events_cfg = match &cfg.app.events {
        Some(ev) => ev,
        None => {
            return HttpResponse::InternalServerError().json(types::ErrorResponse {
                error: "Events are not enabled".into(),
            });
        }
    };
    let http_cfg = match &events_cfg.http {
        Some(h) => h,
        None => {
            return HttpResponse::InternalServerError().json(types::ErrorResponse {
                error: "HTTP events are not configured".into(),
            });
        }
    };
    if let Some(hashed_api_key) = &http_cfg.hashed_api_key {
        let api_key_header = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
        let valid = if let Some(api_key) = api_key_header {
            let mut hasher = Sha256::new();
            hasher.update(api_key.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            &hash == hashed_api_key
        } else {
            false
        };
        if !valid {
            return HttpResponse::Unauthorized().json(types::ErrorResponse {
                error: "Invalid API key".into(),
            });
        }
    }
    for event in &payload.events {
        push_event(event.clone()).await;
    }
    HttpResponse::Ok().body("Events ingested")
}
