use crate::values::events::EVENT_CHANNEL;
use actix_web::{HttpRequest, HttpResponse, Responder, web::Bytes};
use futures_util::stream;
use std::time::Duration;
use tokio::time::sleep;

#[derive(serde::Serialize)]
struct SseMessage<T> {
    #[serde(rename = "type")]
    event_type: &'static str,
    data: T,
}

pub async fn sse_handler(_req: HttpRequest) -> impl Responder {
    let rx = EVENT_CHANNEL.subscribe();
    let server_events = stream::unfold(rx, |mut rx| async move {
        let msg = tokio::select! {
            Ok(event) = rx.recv() => {
                SseMessage { event_type: "event", data: event }
            }
            _ = sleep(Duration::from_secs(30)) => {
                SseMessage { event_type: "heartbeat", data: "ping".to_string() }
            }
        };

        let payload = serde_json::to_string(&msg).unwrap();
        let bytes = Bytes::from(format!("{}\n", payload));
        Some((Ok::<Bytes, actix_web::Error>(bytes), rx))
    });
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(server_events)
}
