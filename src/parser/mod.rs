//! Parses responses and does the first level of basic output processing.
//!
//! Receives `Responses` from the `Downloader` and subsequently executes
//! the parsing logic defined in the spider's closure. The parsed data is
//! outputted as a JSON and sent to the Pipeline for further processing.
use std::rc::Rc;

use actix::{Actor, Arbiter, ArbiterService, Context, Handler};
use futures::Future;
use reqwest::Url;
use serde_json::Value;

use crate::crawler::{Item, RequestVec, Response};
pub use crate::parser::page::Page;
use crate::pipeline::Pipeline;
use crate::scheduler::Scheduler;
use crate::settings::{CrawlStrategy, ParserSettings};
use crate::spider::{Condition, ParseRule, Pattern, Spider};

mod page;

#[derive(Default)]
pub struct Parser {
    spider: Rc<Spider>,
}

impl Parser {
    pub fn new(spider: Rc<Spider>) -> Self {
        Self { spider }
    }

    fn process(&self, res: Response) {
        // Construct Page Object from response
        let page = Page::from_response(&res);

        // Urls
        let mut urls = page.urls().clone();

        //
        let mut data: Vec<Value> = Vec::new();

        let crawl_rules = self.spider.crawl_rules();
        for rule in crawl_rules {
            match rule.parse_rule {
                ParseRule::FilterUrls => {
                    urls = Utils::filter_urls(&rule.condition, urls);
                }
                ParseRule::Page(ref parse_rule) => {
                    if let Some(values) = (parse_rule.callback)(&page) {
                        data.extend(values);
                    }
                }
                ParseRule::Pattern(ref parse_rule) => {
                    let urls = Utils::filter_urls(&rule.condition, vec![res.request.url.clone()]);
                    if !urls.is_empty() {
                        let matches = match parse_rule.pattern {
                            Pattern::CssSelector(sel) => page.matches_selectors(sel),
                            Pattern::Regex(exp) => page.matches_regex(exp),
                            Pattern::Xpath(_) => unimplemented!(),
                        };

                        if !matches.is_empty() {
                            if let Some(value) = (parse_rule.callback)(matches) {
                                if data.is_empty() {
                                    data.push(json!({}));
                                }
                                if let Some(data) = data[0].as_object_mut() {
                                    data.insert(parse_rule.field.to_owned(), value);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Set depth of new batch of links
        let depth = res.request.depth + 1;

        // Set priority of new batch of links
        let settings = &self.spider.settings().parser;
        let priority = Utils::calc_priority(settings, &res);

        trace!("Depth: {}   Priority: {}", depth, priority);

        // Send links to scheduler
        send!(Scheduler, RequestVec::from_urls(urls, depth, priority));

        // Send item (json) to pipeline
        for d in data {
            send!(Pipeline, Item::new(res.request.clone(), d));
        }
    }
}

/// Provide Actor implementation for Parser
impl Actor for Parser {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Parser is started");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Parser is stopped");
    }
}

impl actix::Supervised for Parser {}

impl ArbiterService for Parser {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

impl Handler<Response> for Parser {
    type Result = ();

    fn handle(&mut self, msg: Response, _ctx: &mut Context<Self>) {
        trace!("Response: {}", msg.request.url);
        self.process(msg);
    }
}

struct Utils;

impl Utils {
    fn filter_urls(cnd: &Condition, urls: Vec<Url>) -> Vec<Url> {
        urls.into_iter()
            .filter(|url| cnd.allow.is_match(url.as_str()) && !cnd.deny.is_match(url.as_str()))
            .collect()
    }

    fn calc_priority(settings: &ParserSettings, res: &Response) -> u32 {
        let depth = res.request.depth as f32;
        let priority = match settings.crawl_strategy {
            CrawlStrategy::BFO => 1.0 - depth / (depth + 1.0),
            CrawlStrategy::DFO => depth / (depth + 1.0),
            CrawlStrategy::Basic => 1.0,
        };
        // Priority must be integer
        (priority * 1_000_000_000.0) as u32
    }
}
