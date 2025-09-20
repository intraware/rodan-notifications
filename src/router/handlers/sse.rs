use actix_web::rt::time::interval;
use actix_web::web::Bytes;
use actix_web::{HttpRequest, HttpResponse};
use futures_util::stream;
use std::time::Duration;

pub async fn sse_handler(_req: HttpRequest) -> impl actix_web::Responder {
    let ticker = interval(Duration::from_secs(2));
    let server_events = stream::unfold((ticker, 0), |(mut ticker, count)| async move {
        ticker.tick().await;
        let payload = format!("data: ping {}\n", count);
        let bytes = Bytes::from(payload);
        Some((
            Ok::<actix_web::web::Bytes, actix_web::error::Error>(bytes),
            (ticker, count + 1),
        ))
    });
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(server_events)
}
