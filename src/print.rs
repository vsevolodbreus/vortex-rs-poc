//! Utility for outputting logs to console while `Vortex` is running.
//!
//! `Print` uses both the Downloader Middleware and Pipeline Element functionality
//! to add debug verbosity to standard logging module that `Vortex` uses.
use reqwest::r#async::{ClientBuilder, RequestBuilder};
use serde_json::Value;

use crate::crawler::{Request, Response};
use crate::crawler::Item;
use crate::downloader::middleware::DownloaderMiddleware;
use crate::pipeline::elements::PipelineElement;
use crate::settings::PrintSettings;

/// Downloader Middleware and Pipeline Element that implements output to console
/// functionality
pub struct Print {
    max_len: usize,
}

impl Print {
    #[allow(dead_code)]
    pub fn new(max_len: usize) -> Self {
        Self { max_len }
    }

    pub fn from_settings(settings: PrintSettings) -> Self {
        Self { max_len: settings.max_len }
    }
}

impl DownloaderMiddleware for Print {
    fn process_client(&self, cln: ClientBuilder, _req: &Request) -> ClientBuilder {
        info!("{:?}", cln);
        cln
    }

    fn process_request(&self, req: RequestBuilder) -> RequestBuilder {
        info!("{:?}", req);
        req
    }

    fn process_response(&self, res: Response) -> Response {
        let mut res_clone = res.clone();
        if self.max_len > 0 {
            res_clone.body = Utils::crop_len(res_clone.body.as_str(), self.max_len);
        }
        info!("{:?}", res_clone);
        res
    }
}

impl PipelineElement for Print {
    fn process_item(&self, item: Item) -> Item {
        let mut item_clone = item.clone();
        if self.max_len > 0 {
            if let Some(data) = item_clone.data.as_object_mut() {
                for s in data.clone() {
                    if let Some(v) = s.1.as_str() {
                        data.insert(s.0, Value::String(Utils::crop_len(v, self.max_len)));
                    }
                }
                item_clone.data = Value::Object(data.clone());
            }
        }
        info!("{:?}", item_clone);
        item
    }
}

struct Utils;

impl Utils {
    fn crop_len(src: &str, len: usize) -> String {
        if src.len() > len {
            format!("{}...({})", &src[0..len], src.len() - len)
        } else {
            src.to_string()
        }
    }
}
