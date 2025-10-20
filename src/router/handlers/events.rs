use crate::{
    responses::types::EventResponse,
    utils::events::{Event, get_events},
};
use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};

pub async fn events_get_handler(req: HttpRequest) -> impl Responder {
    let since_time: Option<DateTime<Utc>> = match req.headers().get("Last-Received-Update") {
        Some(header_value) => match header_value.to_str() {
            Ok(value_str) => match DateTime::parse_from_rfc3339(value_str) {
                Ok(parsed) => Some(parsed.with_timezone(&Utc)),
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid Last-Received-Update header");
                }
            },
            Err(_) => {
                return HttpResponse::BadRequest().body("Invalid Last-Received-Update header");
            }
        },
        None => None,
    };
    let events: Vec<Event> = get_events(since_time).await;
    let response: Vec<EventResponse> = events.into_iter().map(EventResponse::from).collect();
    HttpResponse::Ok().json(response)
}
