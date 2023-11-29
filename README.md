# vortex-rs-poc
A proof-of-concept web crawling platform built on Rust. Inspired by Scrapy's (http://scrapy.org) design.

## Design
Vortex is built using system of actors ([actix-core](http://actix.rs)) and consists of the following components:
- Crawler
- Spider
- Scheduler
- Downloader
- Parser
- Pipeline

#### Crawler
The `crawler.rs` file serves as the crawler's entry point, by launching the actix system loop.

#### Spider
Defines a scraping template that must be filled out for a particular source (see examples). The template's parameters include:
- `start_urls` supply a url or a list of urls to initiate the crawl
- `crawl_rules` define which links need to be followed and which need to be parsed, by supplying the parsing logic in a closure

#### Scheduler
Enqueues urls to crawl based on crawling logic priority. Keeps track of crawled urls, prioritizes queue based on crawler settings, including
- Breadth First Order (BFO)
- Depth First Order (DFO)
- Downloader feedback

#### Downloader
Takes care of network resource retrieval. The `Downloader` is fed requests from the `Scheduler` and sends back the respective responses, coupled with the data to the parser. Additional processing of requests is done by the `Downloader` middleware. Features include:
- Header construction
- User Agent Spoofing
- Proxy use toggle
- Assessment of site response (side down, non-200 responses)
- Autothrottle

#### Parser
Receives `Responses` from the `Downloader` and subsequently executes the parsing logic defined in the spider's closure. The parsed data is outputted as a JSON and sent to the Pipeline for further processing.

#### Pipeline
Once an object is scraped, it is sent to the Pipeline. The Pipeline defines post processing logic and routines. Custom post processing logic and be written based on a template and called in the Pipeline. Post processing includes:
- Timestamping
- Redirecting output to a database, search-index
- Formatting output
- Metrics - records scraped, etc
- Filtering
- Classification
- Content Identification
- Index segment preparation? via output redirection to file?
- Other Data Assessment

Eventually ML models would be trained and used in the item pipeline for aforementioned tasks for classification and analysis.


## Building & Running

The examples folder contains examples for:
- wikipedia crawl w/ a TOML file
- wikipedia crawl w/o a TOML file

#### Running from Terminal
From root directory run the following command to compile and launch the program:

```bash
cargo run --example example_name
```

#### Building Spiders
To launch a crawl, you need to build a spider, using the `SpiderBuilder`. The following are some guidelines to get you started on building spiders.

1. Import necessary `vortex` modules
    ```rust
    use vortex::{
        crawler::Crawler,
        settings::Settings,
        spider::{Condition, ParseRule, Pattern, SpiderBuilder},
    };

    ```

2. Initialize logging

    ```rust
    use std::env;
 
    fn main() {
       env::set_var("RUST_LOG", "vortex=info");
       pretty_env_logger::init();
       //...
    }    
    ```

3. Initialize the `SpiderBuilder`. (All subsequent is assumed to be in the `main()` function.)
    
    ```rust
    let mut builder = SpiderBuilder::default(); // TODO: need name?
    ```

4. Give the spider a list of urls to initiate the crawl

    ```rust
    builder.set_start_urls(vec!["http://en.wikipedia.org"]);
    ```
    
5. Set up parsing rules. The most complicated.

    The parsing rules include three different types of rules:
    - Filtering which URLs to follow
    - Defining how to parse the body of a Response of a Request to a particular url
    - Defining how to parse the result of a using a CSS selector or Regex on the Response body and assigning it to a field.
    
6. Override any default settings

    1. Using a TOML file
    2. Directly accessing the settings
    
7. Enabling Middleware

8. Enabling Pipeline elements

9. Build the spider
    
    ```rust
    let spider = builder.build();
    ```

10. Launch the crawler

    ```rust
    Crawler::run(spider);
    ```
