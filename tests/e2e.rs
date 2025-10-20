use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use serial_test::serial;
use sha2::{Digest, Sha256};
use std::env;

fn hash_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key);
    hex::encode(hasher.finalize())
}

async fn ingest_events(client: &Client, host: &str, hashed_key: &str, events: Vec<&str>) {
    let payload = json!({ "events": events });
    let resp = client
        .post(format!("{}/api/events/ingest", host))
        .header("x-api-key", hashed_key)
        .json(&payload)
        .send()
        .await
        .expect("Failed POST /api/events/ingest");
    assert!(resp.status().is_success(), "Failed ingesting events");
}

#[tokio::test]
#[serial]
async fn test_ping() {
    let host = env::var("RODAN_HOST").unwrap_or_else(|_| "http://localhost:8080".into());
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/ping", host))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
#[serial]
async fn test_events_ingest() {
    let host = env::var("RODAN_HOST").unwrap_or_else(|_| "http://localhost:8080".into());
    let raw_key = env::var("RODAN_API_KEY").unwrap_or_else(|_| "1234567890123456".into());
    let hashed_key = hash_key(&raw_key);

    let client = Client::new();
    ingest_events(
        &client,
        &host,
        &hashed_key,
        vec!["user_logged_in", "file_uploaded"],
    )
    .await;
}

#[tokio::test]
#[serial]
async fn test_events_retrieval() {
    let host = env::var("RODAN_HOST").unwrap_or_else(|_| "http://localhost:8080".into());
    let raw_key = env::var("RODAN_API_KEY").unwrap_or_else(|_| "1234567890123456".into());
    let hashed_key = hash_key(&raw_key);

    let client = Client::new();
    ingest_events(
        &client,
        &host,
        &hashed_key,
        vec!["user_logged_in", "file_uploaded", "notification_sent"],
    )
    .await;

    let events_resp = client
        .get(format!("{}/api/events", host))
        .send()
        .await
        .expect("Failed GET /api/events");
    assert_eq!(events_resp.status().as_u16(), 200);

    let events_body = events_resp.text().await.unwrap();
    assert!(
        events_body.contains("user_logged_in")
            && events_body.contains("file_uploaded")
            && events_body.contains("notification_sent"),
        "Events not found in response: {}",
        events_body
    );
}

#[tokio::test]
#[serial]
async fn test_events_since_last_received_update() {
    let host = env::var("RODAN_HOST").unwrap_or_else(|_| "http://localhost:8080".into());
    let raw_key = env::var("RODAN_API_KEY").unwrap_or_else(|_| "1234567890123456".into());
    let hashed_key = hash_key(&raw_key);

    let client = Client::new();
    ingest_events(&client, &host, &hashed_key, vec!["user_logged_in"]).await;
    let timestamp: DateTime<Utc> = Utc::now();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    ingest_events(&client, &host, &hashed_key, vec!["user_logged_out"]).await;
    let events_resp = client
        .get(format!("{}/api/events", host))
        .header("Last-Received-Update", timestamp.to_rfc3339())
        .send()
        .await
        .expect("Failed GET /api/events with Last-Received-Update");
    assert_eq!(events_resp.status().as_u16(), 200);
    let events_body = events_resp.text().await.unwrap();
    assert!(
        events_body.contains("user_logged_out") && !events_body.contains("user_logged_in"),
        "Filtered events not returned correctly: {}",
        events_body
    );
}
