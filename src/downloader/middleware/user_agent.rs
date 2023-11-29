//! User Agent Middleware
use reqwest::header::{HeaderValue, USER_AGENT};
use reqwest::r#async::RequestBuilder;

use crate::downloader::middleware::DownloaderMiddleware;
use crate::settings::UserAgentSettings;

/// Middleware that the `Downloader` uses to set the User-Agent when constructing `Request`s.
pub struct UserAgent {
    value: String,
}

impl UserAgent {
    pub fn new(value: &str) -> Self {
        Self { value: value.to_string() }
    }

    pub fn from_settings(settings: UserAgentSettings) -> Self {
        Self { value: settings.value }
    }
}

impl DownloaderMiddleware for UserAgent {
    fn process_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(USER_AGENT, HeaderValue::from_str(self.value.as_str()).unwrap())
    }
}
