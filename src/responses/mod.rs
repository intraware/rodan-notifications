pub mod types;

use crate::responses::types::{ErrorResponse, PingResponse};
use actix_web::{
    HttpResponse, Responder,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
};

pub fn unauthorized(req: ServiceRequest, msg: &str) -> ServiceResponse<BoxBody> {
    let resp = HttpResponse::Unauthorized().json(ErrorResponse {
        error: msg.to_string(),
    });
    req.into_response(resp.map_into_boxed_body())
}

pub fn bad_request(req: ServiceRequest, msg: &str) -> ServiceResponse<BoxBody> {
    let resp = HttpResponse::BadRequest().json(ErrorResponse {
        error: msg.to_string(),
    });
    req.into_response(resp.map_into_boxed_body())
}

pub async fn ping_response() -> impl Responder {
    HttpResponse::Ok().json(PingResponse {
        msg: "pong".to_string(),
    })
}
