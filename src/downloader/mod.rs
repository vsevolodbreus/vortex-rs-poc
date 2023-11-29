//! The module that performs all network resource retrieval.
//!
//! The `Downloader` is fed requests from the `Scheduler` and sends back
//! the respective response headers, coupled with the data to the parser.
//! Additional processing of requests is done by the `Downloader` middleware.
//!
//! Features include:
//! - Header construction
//! - User Agent Spoofing
//! - Proxy utilization
//!
//! Other Features not yet included:
//! - Assessment of site response (side down, non-200 responses)
//! - Auto-throttle
use std::cell::RefCell;
use std::io::{Cursor, Read};
use std::rc::Rc;

use actix::{Actor, Arbiter, ArbiterService, Context, Handler, Message, Recipient};
use futures::{Future, Stream};
use reqwest::r#async::ClientBuilder;

use crate::crawler::{Listener, Request, Response};
use crate::parser::Parser;
use crate::spider::Spider;

pub mod middleware;

/// The `Downloader` State
///
/// Contains metrics of processed `Requests`
#[derive(Clone, Debug, Default, Message)]
pub struct State {
    pub request_total: usize,
    pub request_success: usize,
    pub request_error: usize,
}

#[derive(Default)]
struct DownloaderInner {
    state: State,
    state_listeners: Vec<Recipient<State>>,
}

impl DownloaderInner {
    fn add_state_listener(&mut self, recipient: Recipient<State>) {
        self.state_listeners.push(recipient);
    }

    fn dispatch_state(&self) {
        self.state_listeners.iter().for_each(|r| {
            let _ = r.do_send(self.state.clone());
        });
    }

    fn increase_request_total(&mut self) {
        self.state.request_total += 1;
        self.dispatch_state();
    }

    fn increase_request_success(&mut self) {
        self.state.request_success += 1;
        self.dispatch_state();
    }

    fn increase_request_error(&mut self) {
        self.state.request_error += 1;
        self.dispatch_state();
    }
}

#[derive(Default)]
pub struct Downloader {
    spider: Rc<Spider>,
    inner: Rc<RefCell<DownloaderInner>>,
}

impl Downloader {
    pub fn new(spider: Rc<Spider>) -> Self {
        Self {
            spider,
            ..Default::default()
        }
    }

    fn process(&self, req: Request) -> impl Future<Item=(), Error=()> {
        let middleware = self.spider.downloader_middleware();

        // Loop through middleware and configure the ClientBuilder with any custom logic
        // defined in any activated middleware
        let mut cln_builder = ClientBuilder::new();
        for m in middleware {
            cln_builder = m.process_client(cln_builder, &req);
        }

        let client = cln_builder.build().unwrap();

        // Loop through middleware and configure the RequestBuilder with any custom logic
        // defined in any activated middleware
        let mut req_builder = client.get(req.url.clone());
        for m in middleware {
            req_builder = m.process_request(req_builder);
        }

        let response = Rc::new(RefCell::new(Response::new(req)));
        let response_clone = Rc::clone(&response);
        let spider_clone = Rc::clone(&self.spider);
        let inner_clone1 = Rc::clone(&self.inner);
        let inner_clone2 = Rc::clone(&self.inner);

        &self.inner.borrow_mut().increase_request_total();

        req_builder
            .send()
            .and_then(move |res| {
                response.borrow_mut().headers = res.headers().clone();
                res.into_body().concat2()
            })
            .map(move |body| {
                let mut res = String::new();
                match Cursor::new(body).read_to_string(&mut res) {
                    Ok(_) => {
                        response_clone.borrow_mut().body = res;

                        let middleware = spider_clone.downloader_middleware();

                        // Loop through middleware and filter/edit the Response based on any custom
                        // logic defined in any activated middleware
                        let mut response = response_clone.borrow().clone();
                        for m in middleware {
                            response = m.process_response(response);
                        }

                        // Send response to parser
                        send!(Parser, response.clone());

                        inner_clone1.borrow_mut().increase_request_success();
                    }
                    Err(e) => {
                        error!("Read body error: {:?}", e);
                        inner_clone1.borrow_mut().increase_request_error();
                    }
                }
            })
            .map_err(move |e| {
                error!("Request error: {:?}", e);
                inner_clone2.borrow_mut().increase_request_error();
            })
    }
}

/// Provide Actor implementation for `Downloader`
impl Actor for Downloader {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Downloader is started");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Downloader is stopped");
    }
}

impl actix::Supervised for Downloader {}

impl ArbiterService for Downloader {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

/// Define handler for `Listener<State>` message
impl Handler<Listener<State>> for Downloader {
    type Result = ();

    fn handle(&mut self, msg: Listener<State>, _ctx: &mut Context<Self>) {
        self.inner.borrow_mut().add_state_listener(msg.r);
    }
}

/// Define handler for `Request` message
impl Handler<Request> for Downloader {
    type Result = ();

    fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) {
        trace!("Request: {}", msg.url);
        Arbiter::spawn(self.process(msg));
    }
}
