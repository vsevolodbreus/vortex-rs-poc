//! Aggregates performance stats
use actix::{Actor, ArbiterService, Context, Handler};

use crate::downloader;
use crate::scheduler;

#[derive(Default)]
pub struct Stats;

/// Provide Actor implementation for `Stats`
impl Actor for Stats {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Stats is started");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!("Stats is stopped");
    }
}

impl actix::Supervised for Stats {}

impl ArbiterService for Stats {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

/// Define handler for `scheduler::State` message
impl Handler<scheduler::State> for Stats {
    type Result = ();

    fn handle(&mut self, msg: scheduler::State, _ctx: &mut Context<Self>) {
        info!("{:?}", msg);
    }
}

/// Define handler for `downloader::State` message
impl Handler<downloader::State> for Stats {
    type Result = ();

    fn handle(&mut self, msg: downloader::State, _ctx: &mut Context<Self>) {
        info!("{:?}", msg);
    }
}
