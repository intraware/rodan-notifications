// Copyright (c) 2025 Intraware
// Licensed under the MIT License
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://opensource.org/licenses/MIT
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod array;
mod event;
mod logging;
pub use event::Event;

use array::EventArray;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::values::{config::get_config, events::EVENT_CHANNEL};

static GLOBAL_EVENT_ARRAY: Lazy<Arc<RwLock<EventArray>>> = Lazy::new(|| {
    let cfg = get_config();
    let segment_size = cfg.app.event_segment_size.unwrap_or(1000);
    let max_segments = cfg.app.event_max_segments.unwrap_or(10);
    Arc::new(RwLock::new(EventArray::new(segment_size, max_segments)))
});

pub async fn push_event(message: String) {
    let _ = EVENT_CHANNEL.send(message.clone());
    let event = Event {
        timestamp: Utc::now(),
        payload: message,
    };
    let mut arr = GLOBAL_EVENT_ARRAY.write().await;
    arr.append(event).await;
}

pub async fn flush_events() {
    let mut arr = GLOBAL_EVENT_ARRAY.write().await;
    arr.flush_all().await;
}

pub async fn get_events(time: Option<DateTime<Utc>>) -> Vec<Event> {
    let arr = GLOBAL_EVENT_ARRAY.read().await;
    match time {
        Some(since) => arr.query_since(since).await,
        None => arr.query_all().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tokio;

    async fn reset_global_array() {
        let test_arr = EventArray::new(100, 10);
        let mut arr = GLOBAL_EVENT_ARRAY.write().await;
        *arr = test_arr;
    }

    #[tokio::test]
    #[serial]
    async fn test_push_event_adds_to_global_array() {
        reset_global_array().await;
        push_event("Event A".into()).await;
        push_event("Event B".into()).await;
        let arr = GLOBAL_EVENT_ARRAY.read().await;
        let all_events = arr.query_all().await;
        let payloads: Vec<_> = all_events.iter().map(|e| e.payload.clone()).collect();
        assert_eq!(payloads, vec!["Event A", "Event B"]);
    }

    #[tokio::test]
    #[serial]
    async fn test_flush_events_clears_global_array() {
        reset_global_array().await;
        push_event("Event 1".into()).await;
        push_event("Event 2".into()).await;
        {
            let arr = GLOBAL_EVENT_ARRAY.read().await;
            assert_eq!(arr.query_all().await.len(), 2);
        }
        flush_events().await;
        {
            let arr = GLOBAL_EVENT_ARRAY.read().await;
            let events_after = arr.query_all().await;
            assert!(
                events_after.is_empty(),
                "Events should be empty after flush"
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_push_event_multiple_times_and_persist() {
        reset_global_array().await;
        for i in 0..5 {
            push_event(format!("Msg {}", i)).await;
        }
        let arr = GLOBAL_EVENT_ARRAY.read().await;
        let events = arr.query_all().await;
        let payloads: Vec<_> = events.iter().map(|e| e.payload.clone()).collect();
        assert_eq!(payloads, vec!["Msg 0", "Msg 1", "Msg 2", "Msg 3", "Msg 4"]);
    }
}
