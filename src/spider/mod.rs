//! Defines a scraping template that must be filled out either for a particular source
//! (see examples) or with more generic cross domain scraping logic.
//!
//! The template's parameters include:
//! - `start_urls` supply a url or a list of urls to initiate the crawl
//! - `crawl_rules` define which links need to be followed and which need to be parsed,
//! by supplying the parsing logic in a closure
use std::rc::Rc;

use regex::RegexSet;
use serde_json::Value;

use crate::crawler::RequestVec;
use crate::downloader::middleware::{DownloaderMiddleware, Proxy, UserAgent};
use crate::parser::Page;
use crate::pipeline::elements::{PipelineElement, Timestamping};
use crate::print::Print;
use crate::settings::{DownloaderMiddlewareType, PipelineElementType, Settings};

type PageCallback = Rc<Fn(&Page) -> Option<Vec<Value>>>;
type PatternCallback = Rc<Fn(Vec<String>) -> Option<Value>>;

/// Defines the processing logic for URLs:
/// - which ones to continue crawling
/// - which ones to reject
/// - from which ones to extract data
/// - what data to extract
/// - how to process the extracted data
/// - what data to assign to a specified JSON key field
///
/// Contains a `Condition` and a `ParseRule`.
#[derive(Clone)]
pub struct CrawlRule {
    pub condition: Condition,
    pub parse_rule: ParseRule,
}

/// Presents a condition used to filter URLs that is defined by two overlapping regular expressions
/// (RegEx) for ease of use.
///
/// `allow` looks at which URLs to include
/// `deny` looks at which URLs to exclude
///
/// The intersection of these 2 conditions yields the set of URLs that is processed in an instance
/// of a `CrawlRule`.
#[derive(Clone)]
pub struct Condition {
    pub allow: RegexSet,
    pub deny: RegexSet,
}

impl Condition {
    pub fn new(allow: Vec<&'static str>, deny: Vec<&'static str>) -> Self {
        Self {
            allow: RegexSet::new(allow).unwrap(),
            deny: RegexSet::new(deny).unwrap(),
        }
    }
}

/// Presents different options for parsing the `Response` body.
#[derive(Clone)]
pub enum ParseRule {
    /// Don't parse the `Response`. (In this case, the associated `Condition` is used to filter
    /// the urls for crawling)
    FilterUrls,

    /// Use `ParsePage`. Create a custom closure that handles all the logic of parsing and
    /// JSON construction
    Page(ParsePage),

    /// Use `ParsePattern`. Use the provided struct to assign a single JSON field a value.
    Pattern(ParsePattern),
}

impl ParseRule {
    pub fn callback<F: 'static>(callback: F) -> Self
        where
            F: Fn(&Page) -> Option<Vec<Value>>,
    {
        ParseRule::Page(ParsePage {
            callback: Rc::new(callback),
        })
    }

    pub fn pattern<F: 'static>(field: &'static str, pattern: Pattern, callback: F) -> Self
        where
            F: Fn(Vec<String>) -> Option<Value>,
    {
        ParseRule::Pattern(ParsePattern {
            field,
            pattern,
            callback: Rc::new(callback),
        })
    }
}

/// Manually parses the html and returns a JSON
#[derive(Clone)]
pub struct ParsePage {
    /// A custom closure that defines how to process the `Response` body. Use this closure
    /// to manually construct a JSON object from the html and return it.
    pub callback: PageCallback,
}

/// Assigns the output of the `callback` to a `field`.
#[derive(Clone)]
pub struct ParsePattern {
    /// The name of the JSON key to which the output of the `callback` will be assigned
    pub field: &'static str,

    /// A selector or expression that determines which section of the HTML to process in the
    /// `callback`
    pub pattern: Pattern,

    /// A closure that processes the result of applying the `pattern` to a `Response` body.
    pub callback: PatternCallback,
}

/// The available ways of extracting a section from the HTML-tree
#[derive(Clone)]
pub enum Pattern {
    /// Use a CSS Selector
    CssSelector(&'static str),

    /// Use a Regular Expression
    Regex(&'static str),

    /// Use an xpath - NOT IMPLEMENTED!
    Xpath(&'static str),
}

/// Used to construct a `Spider`
#[derive(Default)]
pub struct SpiderBuilder {
    /// The URLs to initiate the crawl
    start_requests: RequestVec,

    /// The settings used for the crawl
    settings: Settings,

    /// The rules for filtering URLs and parsing `Responses`
    crawl_rules: Vec<CrawlRule>,

    /// Enabled `middleware` in the `downloader` for `Request` modification
    middleware: Vec<Box<dyn DownloaderMiddleware>>,

    /// Enabled `pipeline` elements for post-processing
    elements: Vec<Box<dyn PipelineElement>>,
}

impl SpiderBuilder {
    /// Set `Spider` name
    pub fn name(mut self, name: &str) -> Self {
        self.settings.spider.name = name.to_string();
        self
    }

    /// Set `Spider` version
    pub fn version(mut self, version: &str) -> Self {
        self.settings.spider.version = version.to_string();
        self
    }

    /// Construct a `RequestVec` from a `Vec` of URL strings
    pub fn start_urls(mut self, urls: Vec<&str>) -> Self {
        self.start_requests = RequestVec::from_strs(urls, 0, 1);
        self
    }

    /// Set the settings parameters.
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    /// Add a crawl rule
    pub fn crawl_rule(mut self, condition: Condition, parse_rule: ParseRule) -> Self {
        self.crawl_rules.push(CrawlRule { condition, parse_rule });
        self
    }

    /// Enable a `downloader` middleware
    pub fn downloader_middleware<T: 'static>(mut self, middleware: T) -> Self
        where T: DownloaderMiddleware
    {
        self.middleware.push(Box::new(middleware));
        self
    }

    /// Enable a `pipeline` element
    pub fn pipeline_element<T: 'static>(mut self, pipeline: T) -> Self
        where T: PipelineElement
    {
        self.elements.push(Box::new(pipeline));
        self
    }

    /// Final step in building a `Spider`. This will consume your `SpiderBuilder` and
    /// return a `Spider` will all parameters and instructions set for use in the crawler.
    pub fn build(mut self) -> Spider {
        // Add middleware from settings
        let middleware_list = self.settings.downloader.middleware_list.clone();
        for item in middleware_list {
            let middleware: Box<dyn DownloaderMiddleware> = match item {
                DownloaderMiddlewareType::UserAgent => {
                    let settings = self.settings.downloader.middleware.user_agent.clone();
                    Box::new(UserAgent::from_settings(settings))
                }
                DownloaderMiddlewareType::Proxy => {
                    let settings = self.settings.downloader.middleware.proxy.clone();
                    Box::new(Proxy::from_settings(settings))
                }
                DownloaderMiddlewareType::Print => {
                    let settings = self.settings.downloader.middleware.print.clone();
                    Box::new(Print::from_settings(settings))
                }
            };
            self.middleware.push(middleware);
        }

        // Add pipeline from settings
        let element_list = self.settings.pipeline.element_list.clone();
        for item in element_list {
            let pipeline: Box<dyn PipelineElement> = match item {
                PipelineElementType::Timestamping => {
                    let settings = self.settings.pipeline.element.timestamping.clone();
                    Box::new(Timestamping::from_settings(settings))
                }
                PipelineElementType::Print => {
                    let settings = self.settings.pipeline.element.print.clone();
                    Box::new(Print::from_settings(settings))
                }
            };
            self.elements.push(pipeline);
        }

        Spider {
            start_requests: self.start_requests,
            settings: self.settings,
            crawl_rules: self.crawl_rules,
            middleware: self.middleware,
            elements: self.elements,
        }
    }
}

/// Contains the unique parameters and instructions that define everything for a crawl. The
/// `Spider` is constructed and then returned from the `SpiderBuilder`.
#[derive(Default)]
pub struct Spider {
    /// The URLs to initiate the crawl
    start_requests: RequestVec,

    /// The settings used for the crawl
    settings: Settings,

    /// The rules for filtering URLs and parsing `Responses`
    crawl_rules: Vec<CrawlRule>,

    /// Enabled `middleware` in the `downloader` for `Request` modification
    middleware: Vec<Box<dyn DownloaderMiddleware>>,

    /// Enabled `pipeline` elements for post-processing
    elements: Vec<Box<dyn PipelineElement>>,
}

impl Spider {
    /// Get `Spider` name
    pub fn name(&self) -> &str {
        &self.settings.spider.name
    }

    /// Get `Spider` version
    pub fn version(&self) -> &str {
        &self.settings.spider.version
    }

    /// Get the start `Requests` as a `RequestVec`
    pub fn start_requests(&self) -> &RequestVec {
        &self.start_requests
    }

    /// Get the `settings`
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get a reference to the crawl rules
    pub fn crawl_rules(&self) -> &Vec<CrawlRule> {
        &self.crawl_rules
    }

    /// Get a reference to the enabled `downloader` middleware
    pub fn downloader_middleware(&self) -> &Vec<Box<dyn DownloaderMiddleware>> {
        &self.middleware
    }

    /// Get a reference to the enabled `pipeline` elements
    pub fn pipeline_elements(&self) -> &Vec<Box<dyn PipelineElement>> {
        &self.elements
    }
}
