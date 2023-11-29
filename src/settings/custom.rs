///??
use std::{fs::File, io, io::Read, path::Path};

use toml;

use crate::settings::{
    CrawlStrategy, DownloaderMiddlewareType, PipelineElementType, PrintSettings,
    ProxySettings, TimestampingSettings, UserAgentSettings,
};

///?? Main `Settings` by module
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// `Spider` settings
    pub spider: Option<SpiderSettings>,

    /// `Scheduler` settings
    pub scheduler: Option<SchedulerSettings>,

    /// `Downloader` settings
    pub downloader: Option<DownloaderSettings>,

    /// `Parser` settings
    pub parser: Option<ParserSettings>,

    /// `Pipeline` settings
    pub pipeline: Option<PipelineSettings>,
}

impl Settings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let s = Settings::read_to_string(path).unwrap();
        toml::from_str(s.as_str()).unwrap()
    }

    fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
}

/// `Spider` settings
#[derive(Clone, Debug, Deserialize)]
pub struct SpiderSettings {
    /// Name of the spider template. Name needs to be set to run a spider.
    pub name: Option<String>,

    /// Spider version
    pub version: Option<String>,
}

/// `Scheduler` settings
#[derive(Clone, Debug, Deserialize)]
pub struct SchedulerSettings {
    /// Delay between issuing `Requests` to the `Downloader`
    pub download_delay: Option<u64>,

    /// Quantity of `Requests` being sent in parallel to the `Downloader`
    pub concurrent_requests: Option<usize>,
}

/// `Downloader` settings
#[derive(Clone, Debug, Deserialize)]
pub struct DownloaderSettings {
    /// List of available middleware modules
    pub middleware_list: Option<Vec<DownloaderMiddlewareType>>,

    /// `Downloader` Middleware settings
    pub middleware: Option<DownloaderMiddlewareSettings>,
}

///?? `Downloader` Middleware settings by module
#[derive(Clone, Debug, Deserialize)]
pub struct DownloaderMiddlewareSettings {
    /// Proxy module settings
    pub proxy: Option<ProxySettings>,

    /// User Agent module settings
    pub user_agent: Option<UserAgentSettings>,

    /// Print module settings
    pub print: Option<PrintSettings>,
}

/// `Parser` settings
#[derive(Clone, Debug, Deserialize)]
pub struct ParserSettings {
    /// Crawl strategies
    pub crawl_strategy: Option<CrawlStrategy>,
}

/// `Pipeline` settings
#[derive(Clone, Debug, Deserialize)]
pub struct PipelineSettings {
    /// List of available pipeline elements
    pub pipeline_list: Option<Vec<PipelineElementType>>,

    /// `Pipeline` Element settings
    pub element: Option<PipelineElementSettings>,
}

/// `Pipeline` Element settings
#[derive(Clone, Debug, Deserialize)]
pub struct PipelineElementSettings {
    /// Timestamping module settings
    pub timestamping: Option<TimestampingSettings>,

    /// Print module settings
    pub print: Option<PrintSettings>,
}
