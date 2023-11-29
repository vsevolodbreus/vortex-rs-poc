//! Wikipedia Example 2: Basic Article Crawler
//!
//! This crawler starts at the main english wiki page and crawls until terminated
use std::env;

use serde_json::Value;

use vortex::{
    crawler::Crawler,
    downloader::middleware::UserAgent,
    pipeline::elements::Timestamping,
    print::Print,
    settings::Settings,
    spider::{Condition, ParseRule, Pattern, SpiderBuilder},
};

fn main() {
    // Initialize logger
    env::set_var("RUST_LOG", "vortex=info");
    pretty_env_logger::init();

    // Specify a condition - a combo of regex expressions for:
    // - which URLs to consider: allow
    // - which URLs to filter out: deny
    let cnd = Condition::new(
        vec![r"en.wikipedia.org/wiki"],
        vec![r":[A-Za-z]|\?|#"],
    );

    // Set spider-specific settings, by overriding the default values defined in [`Settings`]
    let mut settings = Settings::default();
    settings.scheduler.download_delay = 200;

    // Create a new instance of a spider builder
    let spider = SpiderBuilder::default()

        // Define a vector of start urls that the spider will use to initiate crawling
        .start_urls(vec!["http://en.wikipedia.org"])

        // Initial crawl rule filters out all urls that don't satisfy
        // the condition
        .crawl_rule(
            cnd.clone(),
            ParseRule::FilterUrls,
        )

        // Add a crawl rule for the 'title' field.
        // Use a CSS selector to extract the field from the HTML and
        // a close to return the extracted text inside of a json 'Value' type
        .crawl_rule(
            cnd.clone(),
            ParseRule::pattern(
                "title",
                Pattern::CssSelector(".firstHeading"),
                |s| {
                    Some(Value::String(s.first().unwrap().clone()))
                }))

        // Add a crawl rule for the 'categories' field
        // Use a CSS selector to extract the field from the HTML and
        // a closure to return the extracted text inside of a json 'Value' type
        .crawl_rule(
            cnd.clone(),
            ParseRule::pattern(
                "categories",
                Pattern::CssSelector("#mw-normal-catlinks a[href*='/wiki/Category:']"),
                |s| {
                    Some(Value::Array(s.iter()
                        .map(|c| { Value::String(c.to_string()) })
                        .collect()))
                }))

        // Add settings
        .settings(settings)

        // Add User Agent Middleware
        .downloader_middleware(UserAgent::new("Mozilla/5.0"))

        // Add a Timestamp to output Items
        .pipeline_element(Timestamping::default())

        // Add Print Pipeline element to display Items, limiting all
        // field lengths to 100 chars
        .pipeline_element(Print::new(100))

        // Build spider
        .build();

    // Run crawler, initialized with spider
    Crawler::run(spider);
}
