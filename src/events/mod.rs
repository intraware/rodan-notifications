// TODO: add channel for notifying when pushed
use std::collections::VecDeque;

#[derive(Debug)]
struct Queue<T> {
    items: VecDeque<T>,
    capacity: usize,
}

impl<T> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            Err(item)
        } else {
            self.items.push_back(item);
            Ok(())
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }
}

pub struct EventQueue<T> {
    queues: Vec<Queue<T>>,
    queue_capacity: usize,
    front_index: usize,
}

impl<T: Clone> EventQueue<T> {
    pub fn new(queue_capacity: usize) -> Self {
        Self {
            queues: Vec::new(),
            queue_capacity,
            front_index: 0,
        }
    }
    pub fn push(&mut self, item: T) {
        if let Some(last) = self.queues.last_mut() {
            if last.push(item.clone()).is_ok() {
                return;
            }
        }
        let mut new_queue = Queue::new(self.queue_capacity);
        let _ = new_queue.push(item);
        self.queues.push(new_queue);
    }
    pub fn pop(&mut self) -> Option<T> {
        while self.front_index < self.queues.len() {
            if let Some(item) = self.queues[self.front_index].pop() {
                if self.queues[self.front_index].is_empty() {
                    self.front_index += 1;
                }
                return Some(item);
            } else {
                self.front_index += 1;
            }
        }
        if self.front_index > 10 {
            self.queues.drain(0..self.front_index);
            self.front_index = 0;
        }
        None
    }
    pub fn is_empty(&self) -> bool {
        self.front_index >= self.queues.len()
            || self.queues[self.front_index..].iter().all(|q| q.is_empty())
    }
    pub fn is_full(&self) -> bool {
        match self.queues.last() {
            Some(q) => q.is_full(),
            None => false,
        }
    }
}
