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
    let server_events = stream::unfold((), move |_| {
        let mut rx = rx.resubscribe();
        async move {
            loop {
                tokio::select! {
                    Ok(event) = rx.recv() => {
                        let msg = SseMessage {
                            event_type: "event",
                            data: event,
                        };
                        let payload = serde_json::to_string(&msg).unwrap();
                        let bytes = Bytes::from(format!("{}\n", payload));
                        return Some((Ok::<Bytes, actix_web::Error>(bytes), ()));
                    }
                    _ = sleep(Duration::from_secs(30)) => {
                        let msg = SseMessage {
                            event_type: "heartbeat",
                            data: "ping",
                        };
                        let payload = serde_json::to_string(&msg).unwrap();
                        let bytes = Bytes::from(format!("{}\n", payload));
                        return Some((Ok::<Bytes, actix_web::Error>(bytes), ()));
                    }
                }
            }
        }
    });

    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(server_events)
}
