use crate::responses::unauthorized;
use crate::utils::auth;
use actix_web::{
    HttpMessage,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h,
        None => return Ok(unauthorized(req, "Authorization header is required")),
    };
    let auth_str = auth_header.to_str().unwrap_or("");
    if !auth_str.starts_with("Bearer ") {
        return Ok(unauthorized(req, "Bearer token is required"));
    }
    let token = &auth_str[7..];
    let token_data = match auth::decode_jwt(token) {
        Ok(data) => data,
        Err(_) => return Ok(unauthorized(req, "Invalid token")),
    };
    req.extensions_mut().insert(token_data);
    next.call(req).await
}
