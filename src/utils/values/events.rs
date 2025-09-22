use std::sync::{LazyLock, RwLock};

use crate::events::EventQueue;

pub static GLOBAL_EVENT_QUEUE: LazyLock<RwLock<EventQueue<String>>> =
    LazyLock::new(|| RwLock::new(EventQueue::new(100)));

pub fn push_event(event: String) {
    let mut queue = GLOBAL_EVENT_QUEUE.write().unwrap();
    queue.push(event);
}

pub fn pop_event() -> Option<String> {
    let mut queue = GLOBAL_EVENT_QUEUE.write().unwrap();
    queue.pop()
}

pub fn is_event() -> bool {
    let queue = GLOBAL_EVENT_QUEUE.read().unwrap();
    !queue.is_empty()
}
