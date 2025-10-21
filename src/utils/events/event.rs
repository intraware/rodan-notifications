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

use crate::{
    utils::events::logging::{GLOBAL_LOG, Log},
    values::config::get_config,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub payload: String,
}

pub struct EventQueue {
    start_timestamp: Option<DateTime<Utc>>,
    end_timestamp: Option<DateTime<Utc>>,
    pub events: RwLock<Vec<Event>>,
    pub capacity: usize,
    pub log: Option<Arc<Log>>,
}

impl EventQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            start_timestamp: None,
            end_timestamp: None,
            events: RwLock::new(Vec::with_capacity(capacity)),
            capacity,
            log: if get_config().app.event_logging {
                Some(GLOBAL_LOG.clone())
            } else {
                None
            },
        }
    }

    pub async fn push(&mut self, event: Event) -> bool {
        if self.is_full().await {
            return false;
        }
        if self.start_timestamp.is_none() {
            self.start_timestamp = Some(event.timestamp);
        }
        self.end_timestamp = Some(event.timestamp);
        let mut events = self.events.write().await;
        events.push(event);
        true
    }

    pub async fn pop(&mut self) {
        if get_config().app.event_logging {
            self.flush_events().await;
        }
        self.reset().await
    }

    async fn flush_events(&self) {
        if let Some(log) = &self.log {
            let mut events = self.events.write().await;
            for event in events.drain(..) {
                log.add_event(event.payload).await;
            }
            let log_file = get_config().app.events_logfile.clone().unwrap();
            log.write_events(log_file).await;
        }
    }

    pub async fn is_full(&self) -> bool {
        let events = self.events.read().await;
        events.len() >= self.capacity
    }

    pub async fn reset(&mut self) {
        self.start_timestamp = None;
        self.end_timestamp = None;
        let mut events = self.events.write().await;
        events.clear();
    }

    pub fn is_before(&self, time: DateTime<Utc>) -> bool {
        match self.end_timestamp {
            Some(end) => end < time,
            None => true,
        }
    }

    pub async fn get_events(&self, time: Option<DateTime<Utc>>) -> Vec<Event> {
        let events = self.events.read().await;
        if let Some(t) = time {
            events
                .iter()
                .filter(|e| e.timestamp >= t)
                .cloned()
                .collect()
        } else {
            events.iter().cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_event_queue_push_and_is_full() {
        let mut queue = EventQueue::new(2);
        assert!(!queue.is_full().await);
        let event1 = Event {
            timestamp: Utc::now(),
            payload: "Event 1".into(),
        };
        let event2 = Event {
            timestamp: Utc::now(),
            payload: "Event 2".into(),
        };
        assert!(queue.push(event1).await);
        assert!(!queue.is_full().await);
        assert!(queue.push(event2).await);
        assert!(queue.is_full().await);
    }

    #[tokio::test]
    async fn test_event_queue_reset() {
        let mut queue = EventQueue::new(2);
        queue
            .push(Event {
                timestamp: Utc::now(),
                payload: "Event 1".into(),
            })
            .await;
        queue.reset().await;
        let events = queue.events.read().await;
        assert!(events.is_empty());
        assert!(queue.start_timestamp.is_none());
        assert!(queue.end_timestamp.is_none());
    }

    #[tokio::test]
    async fn test_event_queue_flush_to_global_log() {
        let mut queue = EventQueue::new(2);
        queue
            .push(Event {
                timestamp: Utc::now(),
                payload: "Flush Event 1".into(),
            })
            .await;
        queue
            .push(Event {
                timestamp: Utc::now(),
                payload: "Flush Event 2".into(),
            })
            .await;
        queue.pop().await;
        let queue_events = queue.events.read().await;
        assert!(queue_events.is_empty());
        assert!(queue.start_timestamp.is_none());
        assert!(queue.end_timestamp.is_none());
    }
}
