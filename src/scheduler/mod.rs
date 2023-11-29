//! Enqueues urls to crawl based on crawling logic priority.
//!
//! Keeps track of crawled urls, prioritizes queue based on crawler settings.
//!
//! Crawl strategies that define priority, include:
//! - Breadth First Order (BFO)
//! - Depth First Order (DFO)
//! - Downloader feedback
use std::{cell::RefCell, rc::Rc, time::Duration};

use actix::{Actor, Arbiter, ArbiterService, Context, Handler, Message, Recipient};
use chrono::Utc;
use futures::{Future, stream::Stream};
use tokio_timer::Interval;

use crate::crawler::{Listener, RequestVec};
use crate::downloader::{self, Downloader};
use crate::scheduler::queue::{Queue, QueueBuilder};
use crate::settings::{CrawlStrategy, ParserSettings};
use crate::spider::Spider;

mod queue;

///??   - ala `Downloader` State
#[derive(Clone, Debug, Message)]
pub struct State {
    pub queue_len: usize,
}

struct SchedulerInner {
    queue: Box<dyn Queue>,
    unprocessed_requests: usize,
    timestamp: i64,
    state_listeners: Vec<Recipient<State>>,
}

impl Default for SchedulerInner {
    fn default() -> Self {
        Self {
            queue: QueueBuilder::build(CrawlStrategy::Basic),
            unprocessed_requests: 0,
            timestamp: Utc::now().timestamp_millis(),
            state_listeners: Vec::new(),
        }
    }
}

impl SchedulerInner {
    pub fn new(settings: ParserSettings) -> Self {
        let queue = QueueBuilder::build(settings.crawl_strategy);
        Self {
            queue,
            ..Default::default()
        }
    }

    fn add_state_listener(&mut self, recipient: Recipient<State>) {
        self.state_listeners.push(recipient);
    }

    fn dispatch_state(&self) {
        let state = State {
            queue_len: self.queue.len(),
        };
        self.state_listeners.iter().for_each(|r| {
            let _ = r.do_send(state.clone());
        });
    }
}

#[derive(Default)]
pub struct Scheduler {
    spider: Rc<Spider>,
    inner: Rc<RefCell<SchedulerInner>>,
}

impl Scheduler {
    pub fn new(spider: Rc<Spider>) -> Self {
        let inner = Rc::new(RefCell::new(
            SchedulerInner::new(spider.settings().parser.clone())));
        Self { spider, inner }
    }

    fn run_queue_handler(&self) {
        let settings = self.spider.settings().scheduler.clone();
        let inner_clone = Rc::clone(&self.inner);
        Arbiter::spawn(
            Interval::new_interval(Duration::from_millis(settings.download_delay))
                .for_each(move |_| {
                    let timestamp = Utc::now().timestamp_millis();
                    if inner_clone.borrow().unprocessed_requests < settings.concurrent_requests
                        && (timestamp - inner_clone.borrow().timestamp) > settings.download_delay as i64
                    {
                        if let Some(req) = inner_clone.borrow_mut().queue.pop() {
                            send!(Downloader, req);
                        }
                        inner_clone.borrow_mut().timestamp = timestamp;
                        inner_clone.borrow().dispatch_state();
                    }
                    Ok(())
                })
                .map_err(|e| error!("Timer error: {:?}", e)));
    }
}

/// Provide Actor implementation for Scheduler
impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Scheduler is started");
        self.run_queue_handler();
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Scheduler is stopped");
    }
}

impl actix::Supervised for Scheduler {}

impl ArbiterService for Scheduler {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

/// Define handler for `Listener<State>` message
impl Handler<Listener<State>> for Scheduler {
    type Result = ();

    fn handle(&mut self, msg: Listener<State>, _ctx: &mut Context<Self>) {
        self.inner.borrow_mut().add_state_listener(msg.r);
    }
}

/// Define handler for `RequestVec` message
impl Handler<RequestVec> for Scheduler {
    type Result = ();

    fn handle(&mut self, msg: RequestVec, _ctx: &mut Context<Self>) {
        trace!("RequestVec (len): {}", msg.requests.len());
        for req in msg.requests {
            self.inner.borrow_mut().queue.push(req);
        }
        self.inner.borrow().dispatch_state();
    }
}

/// Define handler for `downloader::State` message
impl Handler<downloader::State> for Scheduler {
    type Result = ();

    fn handle(&mut self, msg: downloader::State, _ctx: &mut Context<Self>) {
        self.inner.borrow_mut().unprocessed_requests =
            msg.request_total - msg.request_success - msg.request_error;
    }
}
