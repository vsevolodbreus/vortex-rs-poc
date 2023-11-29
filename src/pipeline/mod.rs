//! Defines post processing logic and routines.
//!
//! Once an object is scraped, it is sent to the Pipeline. Custom post
//! processing logic and be written based on a template and called in
//! the Pipeline.
//!
//! Post processing includes:
//! - Timestamping
//! - Redirecting output to a database, search-index
//! - Formatting output
//! - Metrics - records scraped, etc
//! - Filtering
//! - Classification
//! - Content Identification
//! - Index segment preparation? via output redirection to file?
//! - Other Data Assessment
//!
//! Eventually ML models would be trained and used in the item pipeline
//! for aforementioned tasks for classification and analysis.
use std::rc::Rc;

use actix::{Actor, ArbiterService, Context, Handler};

use crate::crawler::Item;
use crate::spider::Spider;

pub mod elements;

#[derive(Default)]
pub struct Pipeline {
    spider: Rc<Spider>,
}

impl Pipeline {
    pub fn new(spider: Rc<Spider>) -> Self {
        Self { spider }
    }

    fn process(&self, item: Item) {
        let p = self.spider.pipeline_elements();

        let mut item = item.clone();
        for m in p {
            item = m.process_item(item);
        }
    }
}

/// Provide Actor implementation for Pipeline
impl Actor for Pipeline {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Pipeline is started");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Pipeline is stopped");
    }
}

impl actix::Supervised for Pipeline {}

impl ArbiterService for Pipeline {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

impl Handler<Item> for Pipeline {
    type Result = ();

    fn handle(&mut self, msg: Item, _ctx: &mut Context<Self>) {
        trace!("Item: {}", msg.request.url);
        self.process(msg);
    }
}
