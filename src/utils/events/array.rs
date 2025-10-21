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

use crate::utils::events::event::{Event, EventQueue};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EventArray {
    segments: VecDeque<Arc<RwLock<EventQueue>>>,
    segment_capacity: usize,
    max_segments: usize,
}

impl EventArray {
    pub fn new(segment_capacity: usize, max_segments: usize) -> Self {
        Self {
            segments: VecDeque::with_capacity(max_segments),
            segment_capacity,
            max_segments,
        }
    }

    pub async fn append(&mut self, event: Event) {
        if let Some(last_seg) = self.segments.back() {
            let mut seg = last_seg.write().await;
            if seg.push(event.clone()).await {
                return;
            }
        }
        if self.segments.len() >= self.max_segments {
            if let Some(old_seg) = self.segments.pop_front() {
                let mut seg = old_seg.write().await;
                seg.pop().await;
                drop(seg);
                self.segments.push_back(old_seg);
                let mut last_seg = self.segments.back().unwrap().write().await;
                last_seg.push(event).await;
                return;
            }
        }
        let mut new_seg = EventQueue::new(self.segment_capacity);
        new_seg.push(event).await;
        self.segments.push_back(Arc::new(RwLock::new(new_seg)));
    }

    pub async fn query_since(&self, since: DateTime<Utc>) -> Vec<Event> {
        let mut results = Vec::new();

        if self.segments.is_empty() {
            return results;
        }
        let mut left = 0;
        let mut right = self.segments.len() - 1;
        let mut start_index = self.segments.len();
        while left <= right {
            let mid = (left + right) / 2;
            let seg = self.segments[mid].read().await;
            if seg.is_before(since) {
                left = mid + 1;
            } else {
                start_index = mid;
                if mid == 0 {
                    break;
                }
                right = mid - 1;
            }
        }
        for seg_arc in self.segments.iter().skip(start_index) {
            let seg = seg_arc.read().await;
            let events = seg.get_events(Some(since)).await;
            results.extend(events);
        }
        results
    }

    pub async fn query_all(&self) -> Vec<Event> {
        let mut results = Vec::new();
        for seg_arc in &self.segments {
            let seg = seg_arc.read().await;
            let events = seg.get_events(None).await;
            results.extend(events);
        }
        results
    }

    pub async fn flush_all(&mut self) {
        for seg_arc in &self.segments {
            let mut seg = seg_arc.write().await;
            seg.pop().await;
        }
        self.segments.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_append_and_query_all() {
        let mut arr = EventArray::new(2, 3);
        let now = Utc::now();
        let e1 = Event {
            timestamp: now,
            payload: "E1".into(),
        };
        let e2 = Event {
            timestamp: now,
            payload: "E2".into(),
        };
        arr.append(e1.clone()).await;
        arr.append(e2.clone()).await;
        let all = arr.query_all().await;
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].payload, "E1");
        assert_eq!(all[1].payload, "E2");
    }

    #[tokio::test]
    async fn test_segment_rollover() {
        let mut arr = EventArray::new(2, 2);
        for i in 1..=5 {
            arr.append(Event {
                timestamp: Utc::now(),
                payload: format!("E{}", i),
            })
            .await;
        }
        assert_eq!(arr.segments.len(), 2);
        let payloads: Vec<_> = arr
            .query_all()
            .await
            .into_iter()
            .map(|e| e.payload)
            .collect();
        assert_eq!(payloads, vec!["E3", "E4", "E5"]);
    }

    #[tokio::test]
    async fn test_query_since() {
        let mut arr = EventArray::new(2, 3);
        let now = Utc::now();
        let events = vec![
            Event {
                timestamp: now,
                payload: "E1".into(),
            },
            Event {
                timestamp: now + Duration::seconds(10),
                payload: "E2".into(),
            },
            Event {
                timestamp: now + Duration::seconds(20),
                payload: "E3".into(),
            },
        ];
        for e in &events {
            arr.append(e.clone()).await;
        }
        let results = arr.query_since(now + Duration::seconds(10)).await;
        let payloads: Vec<_> = results.into_iter().map(|e| e.payload).collect();
        assert_eq!(payloads, vec!["E2", "E3"]);
    }

    #[tokio::test]
    async fn test_empty_query() {
        let arr = EventArray::new(2, 2);
        assert!(arr.query_all().await.is_empty());
        assert!(arr.query_since(Utc::now()).await.is_empty());
    }

    #[tokio::test]
    async fn test_flush_all() {
        let mut arr = EventArray::new(2, 3);
        let now = Utc::now();
        for i in 1..=3 {
            arr.append(Event {
                timestamp: now,
                payload: format!("Flush{}", i),
            })
            .await;
        }
        let before_flush = arr.query_all().await;
        assert_eq!(before_flush.len(), 3);
        arr.flush_all().await;
        assert!(arr.segments.is_empty());
        assert!(arr.query_all().await.is_empty());
    }

    #[tokio::test]
    async fn test_segment_capacity_limits() {
        let mut arr = EventArray::new(2, 3);
        for i in 1..=6 {
            arr.append(Event {
                timestamp: Utc::now(),
                payload: format!("E{}", i),
            })
            .await;
        }
        assert_eq!(arr.segments.len(), 3);
        let payloads: Vec<_> = arr
            .query_all()
            .await
            .into_iter()
            .map(|e| e.payload)
            .collect();
        assert_eq!(payloads, vec!["E1", "E2", "E3", "E4", "E5", "E6"]);
    }

    #[tokio::test]
    async fn test_query_since_before_all() {
        let mut arr = EventArray::new(2, 2);
        let now = Utc::now();
        arr.append(Event {
            timestamp: now + Duration::seconds(10),
            payload: "E1".into(),
        })
        .await;
        arr.append(Event {
            timestamp: now + Duration::seconds(20),
            payload: "E2".into(),
        })
        .await;
        let results = arr.query_since(now).await;
        let payloads: Vec<_> = results.into_iter().map(|e| e.payload).collect();
        assert_eq!(payloads, vec!["E1", "E2"]);
    }

    #[tokio::test]
    async fn test_query_since_after_all() {
        let mut arr = EventArray::new(2, 2);
        let now = Utc::now();
        arr.append(Event {
            timestamp: now,
            payload: "E1".into(),
        })
        .await;
        arr.append(Event {
            timestamp: now + Duration::seconds(10),
            payload: "E2".into(),
        })
        .await;
        let results = arr.query_since(now + Duration::seconds(20)).await;
        assert!(results.is_empty());
    }
}
