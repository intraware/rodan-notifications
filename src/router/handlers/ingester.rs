use crate::values::events::push_event;
use actix_web::{HttpResponse, Responder, web};

#[derive(serde::Deserialize)]
pub struct EventsPayload {
    pub events: Vec<String>,
}

pub async fn events_ingestor(payload: web::Json<EventsPayload>) -> impl Responder {
    for event in &payload.events {
        push_event(event.clone()).await;
    }
    HttpResponse::Ok().body("Events ingested")
}
