use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn events_ingestor(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("event ingestor")
}
