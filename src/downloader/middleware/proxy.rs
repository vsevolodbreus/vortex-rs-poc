//! Proxy Middleware
use rand::Rng;
use reqwest::{r#async::ClientBuilder, Url};

use crate::crawler::Request;
use crate::downloader::middleware::DownloaderMiddleware;
use crate::settings::ProxySettings;

/// Middleware that defines http and https proxies for the `Downloader` to use
#[derive(Default)]
pub struct Proxy {
    http: Vec<Url>,
    https: Vec<Url>,
}

impl Proxy {
    pub fn from_settings(settings: ProxySettings) -> Self {
        Self {
            http: Utils::strings_to_urls(&settings.http),
            https: Utils::strings_to_urls(&settings.https),
        }
    }

    pub fn add_http(mut self, url: &str) -> Self {
        self.http.push(Url::parse(url).unwrap());
        self
    }

    pub fn add_https(mut self, url: &str) -> Self {
        self.https.push(Url::parse(url).unwrap());
        self
    }
}

impl DownloaderMiddleware for Proxy {
    fn process_client(&self, cln: ClientBuilder, req: &Request) -> ClientBuilder {
        match req.url.scheme() {
            "http" => {
                let i = rand::thread_rng().gen_range(0, self.http.len());
                cln.proxy(reqwest::Proxy::http(self.http[i].clone()).unwrap())
            }
            "https" => {
                let i = rand::thread_rng().gen_range(0, self.https.len());
                cln.proxy(reqwest::Proxy::https(self.https[i].clone()).unwrap())
            }
            _ => cln
        }
    }
}

struct Utils;

impl Utils {
    fn strings_to_urls(src: &Vec<String>) -> Vec<Url> {
        let mut dest = Vec::new();
        for url in src {
            dest.push(Url::parse(url.as_str()).unwrap());
        }
        dest
    }
}
