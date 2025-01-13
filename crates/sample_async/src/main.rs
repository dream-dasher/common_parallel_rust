//! Exploring some tokio init code.
//!
//! ## TLDR
//! - Reqwest
//!   - awkward http request crate
//!   - works with raw strings or a fragile/foot-gunning `Url` crate re-export
//!   - unfortunate necessity
//!     - `httpx` (python) or `ureq` (rust) are similar - string-like operation appears to be the norm
//! - Sqlx
//!   - straight SQL in your rust, but with compile time checking of of Rust types and that database supports the requests
//! - Tokio
//!
//! ## Note
//! **tokio** is not compatible with wasm target.

mod support;
use std::time::Duration;

use reqwest::{Method, Url, header::HeaderMap};
use serde::{Deserialize, Serialize};
use support::{Result, activate_global_default_tracing_subscriber};
use tracing::{Level as L, event as tea};

// #[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
        let _writer_guard = activate_global_default_tracing_subscriber(None, None)?;

        // # `Url`
        // - (sealed) trait `IntoUrl`
        //   - impl for `String` and `&str`
        // - almost everything is a re-export of `Url`
        // - Url ParseError is separate from reqwest::Error
        // - Url is foot-gun heavy
        //   - e.g. whether a path ends in `/` can result in the whole end being stripped
        //   - sequential joins fail in experiments
        //
        const URL_HTTPBIN: &str = "https://httpbin.org";
        const URL_TYPICODE: &str = "https://jsonplaceholder.typicode.com";
        let base_typicode = Url::parse(URL_TYPICODE)?;
        let todos_typicode = base_typicode.join("/todos")?;
        let base_httpbin = Url::parse(URL_HTTPBIN)?;
        let delay_httpbin = base_httpbin.join("/delay/")?;
        let json_httpbin = base_httpbin.join("/json")?;
        tea!(L::DEBUG, ?base_httpbin, ?base_typicode, ?todos_typicode, ?delay_httpbin, ?json_httpbin);
        tea!(L::INFO, a_url=?json_httpbin.as_str());

        // # header-module
        // - HeaderName
        // - HeaderMap
        // - various constant header names available, but don't seem necessary
        // - client can be builg with default_headers for general use
        let default_headers = {
                let mut headers = HeaderMap::new();
                headers.insert("Accept", "application/json".parse().unwrap());
                headers.insert("User-Agent", "rust-reqwest-client".parse().unwrap());
                headers
        };

        // # `Client`
        // - prefer `::builder()`
        //   - alt: `::new()` is effectively `::default()`
        //   - see ClientBuilder
        // - `Arc` used internally
        // - holds an internal connection pool
        let client = reqwest::Client::builder()
                .https_only(true) // this will error for `http` (WARN: not compile-time checked)
                .use_rustls_tls()
                .default_headers(default_headers)
                .timeout(Duration::from_secs(30)) // default is *no* timeout
                .build()?;

        // # `Request`
        // see RequestBuilder
        let request = client
                .request(Method::GET, base_httpbin.join("/headers")?)
                .header("Authorization", "prettyplease")
                .header("Fanciful", "ladeeda")
                .query(&[("query_key", "query_value")])
                .body("the exact body that is sent")
                .build()?;
        tea!(L::DEBUG, ?request);

        // # Response:
        // - status
        //   - error_for_status
        // - cookies
        // - url
        // - text
        // - json
        let response = client.execute(request).await?;
        tea!(L::DEBUG, ?response);
        tea!(L::INFO, resp_status=?response.status());

        // # JSON, typed
        // (see struct `Todo` below)
        {
                let response = client
                        .request(Method::GET, base_typicode.join("/todos")?)
                        .query(&[("userId", "1"), ("completed", "false")])
                        .send()
                        .await?;

                let todos_type_matched: Vec<Todo> = response.json().await?;
                tea!(L::INFO, "Retrieved {} todos", todos_type_matched.len());
                tea!(L::DEBUG, two_todos = ?todos_type_matched.get(0..=1));
                tea!(L::DEBUG, ?todos_type_matched);
                println!("-------,\nFirst Todo, Type-Matched:\n{:#?}\n-------", todos_type_matched.first());
        }
        // # JSON, ad hoc
        {
                let response = client
                        .request(Method::GET, base_typicode.join("/todos")?)
                        .query(&[("userId", "1"), ("completed", "false")])
                        .send()
                        .await?;

                let todos_ad_hoc: serde_json::Value = response.json().await?;
                tea!(L::DEBUG, ?todos_ad_hoc);
                println!("-------,\nFirst Todo, Ad Hoc Construction:\n{:#?}\n-------", todos_ad_hoc.get(0));
        }

        // # Parallel requests with `Futures`
        {
                use futures::future;

                let urls = vec![
                        base_httpbin.join("/delay/3")?,
                        base_httpbin.join("/delay/2")?,
                        base_httpbin.join("/delay/3")?,
                        base_httpbin.join("/delay/2")?,
                        base_httpbin.join("/delay/3")?,
                        base_httpbin.join("/delay/2")?,
                        base_httpbin.join("/delay/3")?,
                        base_httpbin.join("/delay/2")?,
                        base_httpbin.join("/delay/3")?,
                        base_httpbin.join("/delay/2")?,
                ];

                // ## `futures::join_all`
                // - single future gives a single return of a vec (e.g. of results)
                // - no concurrency limit
                // - held up by longest running component future
                // - will hold onto memory for all futures whiel it waits
                // - for large requests it will use `FuturedOrdered` (instead of Unordered)
                {
                        let futures: Vec<_> = urls
                                .iter()
                                .map(|url| client.request(Method::GET, url.clone()).send())
                                .collect();

                        let start_timer = std::time::Instant::now();
                        let results = future::join_all(futures).await;
                        for result in results.iter() {
                                tea!(L::DEBUG, ?result);
                        }
                        let time_passed = start_timer.elapsed();
                        tea!(L::DEBUG, "All requests completed in {:?}", time_passed);
                        println!(
                                "`Join_All`: {} results returned, each with a delay of 2 or 3 seconds, in a total of {} seconds.",
                                results.len(),
                                time_passed.as_secs_f64()
                        );
                }
                // ## `futures::stream`
                // Parallel requests with option for max concurrent requests
                {
                        use futures::stream::{self, StreamExt};

                        const BUFFER_SIZE: usize = 2;
                        let start_timer = std::time::Instant::now();
                        let mut count = 0;
                        let stream = stream::iter(urls)
                                .map(|url| client.get(url).send())
                                // .boxed()
                                .buffer_unordered(BUFFER_SIZE) // Only 2 requests in flight at once
                                // .buffered(2) // yields responses only in the order futures were arranged
                                .collect::<Vec<_>>()
                                .await;

                        for res in stream {
                                count += 1;
                                tea!(L::DEBUG, ?res);
                        }
                        let time_passed = start_timer.elapsed();
                        tea!(L::DEBUG, "All requests completed in {:?}", time_passed);
                        println!(
                                "`Stream.buffer_unordered({})`: {} results streamed back, each with a delay of 2 or 3 seconds, in a total of {} seconds.",
                                BUFFER_SIZE,
                                count,
                                time_passed.as_secs_f64()
                        );
                }
        }
        Ok(())
}

/// Struct to pull typicode responses into
/// Example of using 'typed' JSON with Serde
#[derive(Debug, Serialize, Deserialize)]
struct Todo {
        #[serde(rename = "userId")]
        user_id:   i32,
        id:        i32,
        title:     String,
        completed: bool,
}

// #[cfg(target_arch = "wasm32")]
// fn main() {}
