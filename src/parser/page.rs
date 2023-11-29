//!
use kuchiki::{NodeRef, traits::*};
use regex::Regex;
use reqwest::{Url, UrlError};

use crate::crawler::Response;

///??
pub struct Page {
    doc: NodeRef,
    urls: Vec<Url>,
}

impl Page {
    pub fn from_response(res: &Response) -> Self {
        //??
        let doc = kuchiki::parse_html().one(res.body.as_str());

        //??
        let urls = Utils::get_urls(&doc).iter()
            .filter_map(|url| {
                Utils::normalize_url(&res.request.url, url.as_str()).ok()
            })
            .collect();

        Self { doc, urls }
    }

    pub fn doc(&self) -> &NodeRef {
        &self.doc
    }

    pub fn urls(&self) -> &Vec<Url> {
        &self.urls
    }

    pub fn matches_selectors(&self, sel: &str) -> Vec<String> {
        self.doc.select(sel).unwrap()
            .map(|n| { n.text_contents() })
            .collect()
    }

    pub fn matches_regex(&self, exp: &str) -> Vec<String> {
        Regex::new(exp).unwrap()
            .find_iter(self.doc.to_string().as_str())
            .map(|m| { m.as_str().to_string() })
            .collect()
    }
}

struct Utils;

impl Utils {
    fn get_urls(doc: &NodeRef) -> Vec<String> {
        doc.select("a").unwrap()
            .filter_map(|node| {
                node.as_node().as_element()
                    .and_then(|element| {
                        element.attributes.borrow().get("href")
                            .map(|url| url.to_string())
                    })
            })
            .collect()
    }

    fn normalize_url(src: &Url, url: &str) -> Result<Url, UrlError> {
        // Join with Response source url if relative to create an absolute url
        src.join(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        let base = Url::parse("http://en.wikipedia.org/src/").unwrap();

        let p = Utils::normalize_url(&base, "picture.jpg").unwrap();
        assert_eq!(p.as_str(), "http://en.wikipedia.org/src/picture.jpg");

        let p = Utils::normalize_url(&base, "../picture.jpg").unwrap();
        assert_eq!(p.as_str(), "http://en.wikipedia.org/picture.jpg");

        let p = Utils::normalize_url(&base, "images/picture.jpg").unwrap();
        assert_eq!(p.as_str(), "http://en.wikipedia.org/src/images/picture.jpg");

        let p = Utils::normalize_url(&base, "/images/picture.jpg").unwrap();
        assert_eq!(p.as_str(), "http://en.wikipedia.org/images/picture.jpg");

        let p = Utils::normalize_url(&base, "http://ru.wikipedia.org").unwrap();
        assert_eq!(p.as_str(), "http://ru.wikipedia.org/");

        let p = Utils::normalize_url(&base, "http://ru.wikipedia.org/index.html").unwrap();
        assert_eq!(p.as_str(), "http://ru.wikipedia.org/index.html");
    }
}
