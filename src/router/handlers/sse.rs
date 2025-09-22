use crate::values::events::pop_event;
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse, Responder};
use futures_util::stream;
use std::time::Duration;
use tokio::time::{Instant, sleep_until};

pub async fn sse_handler(_req: HttpRequest) -> impl Responder {
    let server_events = stream::unfold(
        Instant::now() + Duration::from_secs(30),
        |mut next_ping| async move {
            loop {
                let mut events = Vec::new();
                while let Some(event) = pop_event() {
                    events.push(event);
                }
                if !events.is_empty() {
                    let payload = events
                        .into_iter()
                        .map(|e| format!("data: {}\n\n", e))
                        .collect::<String>();
                    let bytes = Bytes::from(payload);
                    return Some((Ok::<Bytes, actix_web::Error>(bytes), next_ping));
                }
                let now = Instant::now();
                if now >= next_ping {
                    next_ping = now + Duration::from_secs(30);
                    let bytes = Bytes::from("data: ping\n\n");
                    return Some((Ok::<Bytes, actix_web::Error>(bytes), next_ping));
                }
                let sleep_until_instant = std::cmp::min(next_ping, now + Duration::from_secs(1));
                sleep_until(sleep_until_instant).await;
            }
        },
    );
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(server_events)
}

