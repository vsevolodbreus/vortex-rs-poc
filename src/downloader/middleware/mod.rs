//! Downloader Middleware
//!
//! Define custom functionality for the `Downloader`.
use reqwest::r#async::{ClientBuilder, RequestBuilder};

use crate::crawler::{Request, Response};
pub use crate::downloader::middleware::{proxy::Proxy, user_agent::UserAgent};

mod proxy;
mod user_agent;

/// Trait that defines a middleware that can be used to add additional
/// functionality to the Downloader.
pub trait DownloaderMiddleware {
    /// Exposes a way to adjusts various parameters of the `ClientBuilder`.
    /// Accepts a `ClientBuilder`, applies custom logic to it and returns a new `ClientBuilder`.
    fn process_client(&self, cln: ClientBuilder, _req: &Request) -> ClientBuilder {
        cln
    }

    /// Exposes a way to adjust various parameters of the `RequestBuilder`
    /// Accepts a `RequestBuilder`, applies custom logic to it and returns a new `RequestBuilder`.
    fn process_request(&self, req: RequestBuilder) -> RequestBuilder {
        req
    }

    /// Exposes a way to edit a response before sending it to the `Parser`.
    fn process_response(&self, res: Response) -> Response {
        res
    }
}
