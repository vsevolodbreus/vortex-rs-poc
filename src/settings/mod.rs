//! Global settings that define crawler behavior
use std::path::Path;
use toml;

use crate::pipeline::elements::TimeOffset;

mod custom;

/// Available `middleware` modules for the `Downloader`
#[derive(Clone, Debug, Deserialize)]
pub enum DownloaderMiddlewareType {
    /// Manually set User Agent
    UserAgent,

    /// Route all requests through a proxy
    Proxy,

    /// Custom print objects for debugging
    Print,
}

/// Predefined crawl strategies
#[derive(Clone, Debug, Deserialize)]
pub enum CrawlStrategy {
    /// Breath First Order - Horizontal priority crawling
    BFO,

    /// Depth First Order - Vertical priority crawling
    DFO,

    /// Arbitrary FIFO - no priority
    Basic,
}

/// Available Modules for `Pipeline` in post processing
#[derive(Clone, Debug, Deserialize)]
pub enum PipelineElementType {
    /// Append custom timestamp to output `Items`
    Timestamping,

    /// Custom print output
    Print,
}

///?? Main `Settings` by module
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// `Spider` settings
    pub spider: SpiderSettings,

    /// `Scheduler` settings
    pub scheduler: SchedulerSettings,

    /// `Downloader` settings
    pub downloader: DownloaderSettings,

    /// `Parser` settings
    pub parser: ParserSettings,

    /// `Pipeline` settings
    pub pipeline: PipelineSettings,
}

impl Default for Settings {
    fn default() -> Self {
        toml::from_str(include_str!("default.toml")).unwrap()
    }
}

impl Settings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Self::default().override_values(custom::Settings::from_file(path))
    }

    pub fn override_values(mut self, settings: custom::Settings) -> Self {
        if let Some(p) = settings.spider {
            self.spider.override_values(p);
        }
        if let Some(p) = settings.scheduler {
            self.scheduler.override_values(p);
        }
        if let Some(p) = settings.downloader {
            self.downloader.override_values(p);
        }
        if let Some(p) = settings.parser {
            self.parser.override_values(p);
        }
        if let Some(p) = settings.pipeline {
            self.pipeline.override_values(p);
        }
        self
    }
}

/// `Spider` settings
#[derive(Clone, Debug, Deserialize)]
pub struct SpiderSettings {
    /// Name of the spider template. Name needs to be set to run a spider.
    pub name: String,

    /// Spider version
    pub version: String,
}

impl SpiderSettings {
    pub fn override_values(&mut self, settings: custom::SpiderSettings) {
        if let Some(v) = settings.name {
            self.name = v;
        }
        if let Some(v) = settings.version {
            self.version = v;
        }
    }
}

/// `Scheduler` settings
#[derive(Clone, Debug, Deserialize)]
pub struct SchedulerSettings {
    /// Delay between issuing `Requests` to the `Downloader`
    pub download_delay: u64,

    /// Quantity of `Requests` being sent in parallel to the `Downloader`
    pub concurrent_requests: usize,
}

impl SchedulerSettings {
    pub fn override_values(&mut self, settings: custom::SchedulerSettings) {
        if let Some(v) = settings.download_delay {
            self.download_delay = v;
        }
        if let Some(v) = settings.concurrent_requests {
            self.concurrent_requests = v;
        }
    }
}

/// `Downloader` settings
#[derive(Clone, Debug, Deserialize)]
pub struct DownloaderSettings {
    /// List of available middleware modules
    pub middleware_list: Vec<DownloaderMiddlewareType>,

    /// `Downloader` Middleware settings
    pub middleware: DownloaderMiddlewareSettings,
}

impl DownloaderSettings {
    pub fn override_values(&mut self, settings: custom::DownloaderSettings) {
        if let Some(v) = settings.middleware_list {
            self.middleware_list = v;
        }
        if let Some(v) = settings.middleware {
            self.middleware.override_values(v);
        }
    }
}

///?? `Downloader` Middleware settings by module
#[derive(Clone, Debug, Deserialize)]
pub struct DownloaderMiddlewareSettings {
    /// Proxy module settings
    pub proxy: ProxySettings,

    /// User Agent module settings
    pub user_agent: UserAgentSettings,

    /// Print module settings
    pub print: PrintSettings,
}

impl DownloaderMiddlewareSettings {
    pub fn override_values(&mut self, settings: custom::DownloaderMiddlewareSettings) {
        if let Some(v) = settings.proxy {
            self.proxy = v;
        }
        if let Some(v) = settings.user_agent {
            self.user_agent = v;
        }
        if let Some(v) = settings.print {
            self.print = v;
        }
    }
}

/// Proxy module settings
#[derive(Clone, Debug, Deserialize)]
pub struct ProxySettings {
    /// A list of http proxies for the `Downloader` to randomly select from
    pub http: Vec<String>,

    ///A list of https proxies for the `Downloader` to randomly select from
    pub https: Vec<String>,
}

/// User Agent module settings
#[derive(Clone, Debug, Deserialize)]
pub struct UserAgentSettings {
    /// User Agent string
    pub value: String,
}

/// Print module settings
#[derive(Clone, Debug, Deserialize)]
pub struct PrintSettings {
    /// The maximum length of a field.
    pub max_len: usize,
}

/// `Parser` settings
#[derive(Clone, Debug, Deserialize)]
pub struct ParserSettings {
    /// Crawl strategies
    pub crawl_strategy: CrawlStrategy,
}

impl ParserSettings {
    pub fn override_values(&mut self, settings: custom::ParserSettings) {
        if let Some(v) = settings.crawl_strategy {
            self.crawl_strategy = v;
        }
    }
}

/// `Pipeline` settings
#[derive(Clone, Debug, Deserialize)]
pub struct PipelineSettings {
    /// List of available pipeline elements
    pub element_list: Vec<PipelineElementType>,

    /// `Pipeline` Element settings
    pub element: PipelineElementSettings,
}

impl PipelineSettings {
    pub fn override_values(&mut self, settings: custom::PipelineSettings) {
        if let Some(v) = settings.pipeline_list {
            self.element_list = v;
        }
        if let Some(v) = settings.element {
            self.element.override_values(v);
        }
    }
}

/// `Pipeline` Element settings
#[derive(Clone, Debug, Deserialize)]
pub struct PipelineElementSettings {
    /// Timestamping module settings
    pub timestamping: TimestampingSettings,

    /// Print module settings
    pub print: PrintSettings,
}

impl PipelineElementSettings {
    pub fn override_values(&mut self, settings: custom::PipelineElementSettings) {
        if let Some(v) = settings.timestamping {
            self.timestamping = v;
        }
        if let Some(v) = settings.print {
            self.print = v;
        }
    }
}

/// Timestamping module settings
#[derive(Clone, Debug, Deserialize)]
pub struct TimestampingSettings {
    ///??
    pub offset: TimeOffset,
    ///??
    pub format: String,
    ///??
    pub field: String,
}
