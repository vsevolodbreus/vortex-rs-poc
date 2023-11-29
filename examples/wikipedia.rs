//! Wikipedia Example 1: Basic Article Crawler
//!
//! This crawler starts at the main english wiki page and crawls until terminated
//!
//! This example uses a TOML file (wikipedia.toml) to set user-specific settings
use std::env;

use serde_json::Value;

use vortex::{
    crawler::Crawler,
    settings::Settings,
    spider::{Condition, ParseRule, Pattern, SpiderBuilder},
};

fn main() {
    // Initialize logger
    env::set_var("RUST_LOG", "vortex=info");
    pretty_env_logger::init();

    // Create a new instance of a spider builder
    let mut builder = SpiderBuilder::default();

    // Initialize the spider with a vector of start urls
    builder = builder.start_urls(vec!["http://en.wikipedia.org"]);

    // Specify a condition - a combo of regex expressions for:
    // - which URLs to consider: allow
    // - which URLs to filter out: deny
    let cnd = Condition::new(
        vec![r"en.wikipedia.org/wiki"],
        vec![r":[A-Za-z]|\?|#"],
    );

    // Initial crawl rule filters out all urls that don't satisfy
    // the condition
    builder = builder.crawl_rule(
        cnd.clone(),
        ParseRule::FilterUrls,
    );

    // Add a crawl rule for the 'title' field.
    // Use a CSS selector to extract the field from the HTML and
    // a closure to return the extracted text inside of a json 'Value' type
    builder = builder.crawl_rule(
        cnd.clone(),
        ParseRule::pattern(
            "title",
            Pattern::CssSelector(".firstHeading"),
            |s| {
                Some(Value::String(s.first().unwrap().clone()))
            }));

    // Add a crawl rule for the 'categories' field
    // Use a CSS selector to extract the field from the HTML and
    // a closure to return the extracted text inside of a json 'Value' type
    builder = builder.crawl_rule(
        cnd.clone(),
        ParseRule::pattern(
            "categories",
            Pattern::CssSelector("#mw-normal-catlinks a[href*='/wiki/Category:']"),
            |s| {
                Some(Value::Array(s.iter()
                    .map(|c| { Value::String(c.to_string()) })
                    .collect()))
            }));


    // Use TOML file in directory to set user settings
    let path = env::current_dir().unwrap().join("examples/wikipedia.toml");
    builder = builder.settings(Settings::from_file(path));

    // Build spider
    let spider = builder.build();

    // Run crawler, initialized with spider
    Crawler::run(spider);
}
