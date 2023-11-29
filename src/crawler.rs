//! The engine component that drives Vortex.
//!
//! The `crawler` module serves as Vortex's entry point, by initializing all the components
//! and launching the actix system loop.
//!
//! The `crawler` also defines all the data types that are used to transfer information
//! between the components (actors).
use std::cmp::Ordering;
use std::rc::Rc;

use actix::{Actor, Addr, Arbiter, dev::ToEnvelope, Handler, Message, Recipient, System};
use reqwest::{header::HeaderMap, Url};
use serde_json::Value;

use crate::downloader::Downloader;
use crate::parser::Parser;
use crate::pipeline::Pipeline;
use crate::scheduler::Scheduler;
use crate::spider::Spider;
use crate::stats::Stats;

/// Contains a `Vec` of `Requests. This is used as the interface to send `Requests`
/// to the `Scheduler`
#[derive(Clone, Default, Debug, Message)]
pub struct RequestVec {
    pub requests: Vec<Request>,
}

impl RequestVec {
    pub fn new(requests: Vec<Request>) -> Self {
        Self { requests }
    }

    pub fn from_strs(urls: Vec<&str>, depth: u32, priority: u32) -> Self {
        let reqs = urls.iter().map(|url| {
            Request::new(Url::parse(url).unwrap(), depth, priority)
        }).collect();
        RequestVec::new(reqs)
    }

    pub fn from_urls(urls: Vec<Url>, depth: u32, priority: u32) -> Self {
        let reqs = urls.iter().map(|url| {
            Request::new(url.clone(), depth, priority)
        }).collect();
        RequestVec::new(reqs)
    }
}

/// Contains the data that is sent to the `Downloader` to make a request to a network resource.
///
/// `Request` also contains priority and depth fields so that the `Scheduler` knows how to
/// establish priority dependencies.
#[derive(Clone, Debug, Message, Eq)]
pub struct Request {
    /// The URL of the request
    pub url: Url,

    /// The distance from the initial `start_urls`. The URLs from the `start_urls`
    /// vector are initialized with a depth of 0. The depth of all URLs derived from parsing
    /// the `Response`s of the `start_urls` URLs is incremented by `1`. Etc.
    pub depth: u32,

    /// The priority is calculated based on the crawling strategy.
    pub priority: u32,
}

impl Ord for Request {
    fn cmp(&self, other: &Request) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Request) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Request) -> bool {
        self.priority == other.priority
    }
}

impl Request {
    pub fn new(url: Url, depth: u32, priority: u32) -> Self {
        Self {
            url,
            depth,
            priority,
        }
    }
}

/// Contains the result of a `Request` fulfilled by the `Downloader`.
#[derive(Clone, Debug, Message)]
pub struct Response {
    /// The `Request` that generated this `Response`.
    pub request: Request,

    /// `Response` headers
    pub headers: HeaderMap,

    /// `Response` body
    pub body: String,
}

impl Response {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            headers: HeaderMap::new(),
            body: String::new(),
        }
    }
}

/// Contains the output of the `Parser` that is sent to the `Pipeline`.
#[derive(Clone, Debug, Message)]
pub struct Item {
    /// The `Request` from which this `Item` has been constructed
    pub request: Request,

    /// A JSON, obtained by assigning the result of CSS-selector or RegEx queries on the
    /// `Response` body to pre-determined fields.
    pub data: Value,
}

impl Item {
    pub fn new(request: Request, data: Value) -> Self {
        Self { request, data }
    }
}

/// An object which implements a subscriber system. It contains the address of an actor
/// which subscribes to state changes of the actor it sends it to.
pub struct Listener<M>
    where
        M: Message + Send,
        M::Result: Send,
{
    pub r: Recipient<M>,
}

impl<M> Listener<M>
    where
        M: Message + Send + 'static,
        M::Result: Send,
{
    pub fn new<A>(addr: Addr<A>) -> Self
        where
            A: Handler<M>,
            A::Context: ToEnvelope<A, M>,
    {
        Self {
            r: addr.recipient::<M>(),
        }
    }
}

impl<T> Message for Listener<T>
    where
        T: Message + Send,
        T::Result: Send,
{
    type Result = ();
}

/// Contains a pointer to a spider template.
///
/// The `Crawler` contains the actix event loop.
pub struct Crawler;

impl Crawler {
    pub fn run(spider: Spider) {
        info!("Run Vortex v{}", env!("CARGO_PKG_VERSION"));

        let sys = System::new("crawler");

        let spider = Rc::new(spider);

        let s = Rc::clone(&spider);
        let scheduler = Scheduler::create(|_| Scheduler::new(s));
        Arbiter::registry().set::<Scheduler>(scheduler.clone());

        let s = Rc::clone(&spider);
        let downloader = Downloader::create(|_| Downloader::new(s));
        Arbiter::registry().set::<Downloader>(downloader.clone());

        let s = Rc::clone(&spider);
        let parser = Parser::create(|_| Parser::new(s));
        Arbiter::registry().set::<Parser>(parser);

        let s = Rc::clone(&spider);
        let pipeline = Pipeline::create(|_| Pipeline::new(s));
        Arbiter::registry().set::<Pipeline>(pipeline);

        let stats = Stats::create(|_| Stats::default());
        Arbiter::registry().set::<Stats>(stats.clone());

        // Add listeners
        scheduler.do_send(Listener::new(stats.clone()));
        downloader.do_send(Listener::new(scheduler.clone()));
        downloader.do_send(Listener::new(stats.clone()));

        // Start point
        scheduler.do_send(spider.start_requests().clone());

        sys.run();
    }
}

/// A macro that sends an asynchronous message to an Actor
macro_rules! send {
    ($actor:path, $msg:expr) => {{
        let addr = Arbiter::registry().get::<$actor>().send($msg);
        Arbiter::spawn(addr.map(|_| {}).map_err(|e| error!("Send error: {:?}", e)));
    }};
}
