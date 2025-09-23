use actix_web::{
    Error,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

pub async fn log_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let res = next.call(req).await?;
    let path = res.request().path();
    let method = res.request().method().as_str();
    let status = res.status().as_u16();
    let ip = res
        .request()
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let log_entry = serde_json::json!({
        "type": "http",
        "status": status,
        "method": method,
        "path": path,
        "ip": ip,
        "level": match status {
            404 => "trace",
            500..=599 => "warn",
            _ => "info",
        }
    });
    println!("{}", log_entry);
    Ok(res)
}
