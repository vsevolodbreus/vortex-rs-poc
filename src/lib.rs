//! # Vortex is a Rust web crawling framework.
//!
//! Vortex can be used to write custom web crawlers for any sort of web crawling task.
//! It can be extended to write robust and large-scale distributed web crawlers,
//! search engines, etc.
//!
//! It is built using an actors framework ([actix](http://actix.rs)) and consists of
//! the following main components:
//! - Crawler
//! - Spider
//! - Scheduler
//! - Downloader
//! - Parser
//! - Pipeline
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[macro_use]
pub mod crawler;
pub mod downloader;
pub mod parser;
pub mod pipeline;
mod scheduler;
pub mod settings;
pub mod spider;
mod stats;
pub mod print;
