//! Defines a queue for the `Scheduler` to use
use std::collections::{BinaryHeap, VecDeque};

use reqwest::Url;

use crate::crawler::Request;
use crate::settings::CrawlStrategy;

/// The `Queue` trait defines 3 basic functions that all queues should implement.
/// Different `std:collections` types are used based on the most efficient
/// queueing mechanism that would work with a given crawl strategy.
///
/// The 3 basic functions are:
/// - push (adding a `Request` to the queue.
/// - pop (retrieving a `Request` from the queue.
/// - len (determining how many `Requests` are in the queue.
pub trait Queue {
    fn push(&mut self, item: Request);
    fn pop(&mut self) -> Option<Request>;
    fn len(&self) -> usize;
}

/// The `QueueBuilder` creates a `Box` pointer that contains the appropriate queue that best fits
/// the selected crawl strategy.
pub struct QueueBuilder;

impl QueueBuilder {
    pub fn build(strategy: CrawlStrategy) -> Box<dyn Queue> {
        match strategy {
            CrawlStrategy::Basic => Box::new(BasicQueue::default()),
            _ => Box::new(PriorityQueue::default()),
        }
    }
}

/// The `BasicQueue` contains 2 vectors that are used to keep track of enqueued and already
/// visited `Request`s.
///
/// `queue` is a double-ended vector (`VecDeque`) that functions as a FIFO. New `Request`s are
/// added at the back-end and processed sequentially from the front-end.
///
/// `visited` is a simple vector that keeps tracks of urls that were already processed by the
/// `downloader`
#[derive(Default)]
struct BasicQueue {
    queue: VecDeque<Request>,
    visited: Vec<Url>,
}

impl Queue for BasicQueue {
    fn push(&mut self, item: Request) {
        if !self.visited.contains(&item.url) {
            self.queue.push_back(item);
        }
    }

    fn pop(&mut self) -> Option<Request> {
        loop {
            match self.queue.pop_front() {
                Some(item) => {
                    if !self.visited.contains(&item.url) {
                        self.visited.push(item.url.clone());
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// The `PriorityQueue` contains 2 vectors that are used to keep track of enqueued and already
/// visited `Request`s.
///
/// `queue` is a `BinarHeap` that sorts the `Request`s based on the priority that the crawl strategy
/// defined.
///
/// `visited` is a simple vector that keeps tracks of urls that were already processed by the
/// `downloader`
#[derive(Default)]
struct PriorityQueue {
    queue: BinaryHeap<Request>,
    visited: Vec<Url>,
}

impl Queue for PriorityQueue {
    fn push(&mut self, item: Request) {
        if !self.visited.contains(&item.url) {
            self.queue.push(item);
        }
    }

    fn pop(&mut self) -> Option<Request> {
        loop {
            match self.queue.pop() {
                Some(item) => {
                    if !self.visited.contains(&item.url) {
                        self.visited.push(item.url.clone());
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_push_pop() {
        let request_1 = Request::new(Url::parse("http://en.wikipedia.org").unwrap(), 0, 1);
        let request_2 = Request::new(Url::parse("http://en.wikipedia.org").unwrap(), 0, 2);
        let request_3 = Request::new(Url::parse("http://ru.wikipedia.org").unwrap(), 1, 1);

        let mut queue = BasicQueue::default();
        queue.push(request_1.clone());
        queue.push(request_2.clone());
        queue.push(request_3.clone());
        assert_eq!(queue.len(), 3);
        let item = queue.pop();
        assert_eq!(item.unwrap().priority, 1);
        assert_eq!(queue.len(), 2);
        queue.push(request_1.clone());
        assert_eq!(queue.len(), 2);
        let item = queue.pop();
        assert_eq!(item.unwrap().depth, 1);
        assert_eq!(queue.pop(), None);

        let mut queue = PriorityQueue::default();
        queue.push(request_1.clone());
        queue.push(request_2.clone());
        queue.push(request_3.clone());
        assert_eq!(queue.len(), 3);
        let item = queue.pop();
        assert_eq!(item.unwrap().priority, 2);
        assert_eq!(queue.len(), 2);
        queue.push(request_1.clone());
        assert_eq!(queue.len(), 2);
        let item = queue.pop();
        assert_eq!(item.unwrap().depth, 1);
        assert_eq!(queue.pop(), None);
    }
}
