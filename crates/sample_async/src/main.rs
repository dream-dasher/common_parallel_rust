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
//! ## Accessories
//! - sites set up to allow http request testing/experimenting
//!   - [htpbin](https://httpbin.org)
//!   - [typicode: jsonplaceholder](https://jsonplaceholder.typicode.com)
//!
//! ## Note
//! **tokio** is not compatible with wasm target.

mod support;
use std::time::Duration;

use reqwest::{Method, Url,
              header::{self, HeaderMap}};
use serde::{Deserialize, Serialize};
use support::{Result, activate_global_default_tracing_subscriber};
use tracing::{debug, error, info, warn};

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
        debug!(?base_httpbin, ?base_typicode, ?todos_typicode, ?delay_httpbin, ?json_httpbin);
        info!(a_url=?json_httpbin.as_str());

        // # Header-module
        // - `HeaderMap`
        //   - key:value map; static hashing for common values
        //   - *multi*map - presumably corresponding to repeated header values (which are in spec)
        //   - re-export from `http` (hyperium)
        //   - many Const values for common headers
        //   - map insertion can panic >32e+3 values
        // - various constant header names available, but don't seem necessary
        // - client can be builg with default_headers for general use
        let default_headers = {
                let mut headers = HeaderMap::new();
                headers.insert(header::ACCEPT, "application/json".parse().unwrap());
                headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
                headers.insert(header::USER_AGENT, "rust-reqwest-client".parse().unwrap());
                // headers.insert(header::AUTHORIZATION, _);
                // headers.insert(header::COOKIE, _);
                // headers.insert(header::DATE, _);
                headers
        };
        debug!(?default_headers);

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
        debug!(?client);

        // # `Request`
        // see RequestBuilder
        let request = client
                .request(Method::GET, base_httpbin.join("/headers")?)
                .header("Authorization", "prettyplease")
                .header("Fanciful", "ladeeda")
                .query(&[("query_key", "query_value")])
                .body("the exact body that is sent")
                .build()?;
        debug!(?request);

        // # Response:
        // - status
        //   - error_for_status
        // - cookies
        // - url
        // - text
        // - json
        let response = client.execute(request).await?;
        debug!(?response);
        info!(resp_status=?response.status());

        // # JSON, typed
        // (see struct `Todo` below)
        {
                let response = client
                        .request(Method::GET, base_typicode.join("/todos")?)
                        .query(&[("userId", "1"), ("completed", "false")])
                        .send()
                        .await?;

                let todos_type_matched: Vec<Todo> = response.json().await?;
                info!("Retrieved {} todos", todos_type_matched.len());
                debug!(two_todos = ?todos_type_matched.get(0..=1));
                debug!(?todos_type_matched);
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
                debug!(?todos_ad_hoc);
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

                        let start_time = std::time::Instant::now();
                        let results = future::join_all(futures).await;
                        for result in results.iter() {
                                debug!(?result);
                        }
                        let time_passed = start_time.elapsed();
                        debug!("All requests completed in {:?}", time_passed);
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
                        let start_time = std::time::Instant::now();
                        let mut count = 0;
                        let mut buffered_stream = stream::iter(urls)
                                .map(|url| client.get(url).send())
                                // .buffered(2) // yields responses only in the order futures were arranged
                                .buffer_unordered(BUFFER_SIZE); // Only 2 requests in flight at once

                        while let Some(result) = buffered_stream.next().await {
                                println!("{}: {:.2}", count, start_time.elapsed().as_secs_f64());
                                count += 1;
                                debug!(?result);
                        }
                        let time_passed = start_time.elapsed();
                        debug!("All requests completed in {:?}", time_passed);
                        println!(
                                "`Stream.buffer_unordered({})`: {} results streamed back, each with a delay of 2 or 3 seconds, in a total of {} seconds.",
                                BUFFER_SIZE,
                                count,
                                time_passed.as_secs_f64()
                        );
                }
        }

        // # `Governor`
        // - single or keyed rate limiting
        // - leaky-bucket ("generic cell rate algorithm")
        // - only takes non-zeroes
        // - appears flexible and nominally quite perofmrant
        // - only takes non-zeroes for quite a few things
        // - `DefaultDirectRateLiminter` & `DefaultKeyedRatelimiter` type aliases to allow easy passing around (!)
        {
                use std::{num::NonZeroU32, sync::Arc};

                use futures::stream::{self, StreamExt};
                use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

                const PER_SECOND_RATE_LIMIT: NonZeroU32 = NonZeroU32::new(12).unwrap();
                const BURST_LIMIT: NonZeroU32 = NonZeroU32::new(6).unwrap();

                let quota = Quota::per_second(PER_SECOND_RATE_LIMIT).allow_burst(BURST_LIMIT);

                // use Default_RateLimiter type alias if you need to refer to these (e.g. in a struct or function)
                let _rate_limiter = RateLimiter::direct(quota);
                let rate_limiter: DefaultDirectRateLimiter = RateLimiter::direct(quota);
                // wrap in an `Arc` to share with various futures/threads
                let arc_rate_limiter = Arc::new(rate_limiter);

                let single_url = [base_httpbin.join("/json")?];

                let start_time = std::time::Instant::now();
                let mut regulated_request_stream = stream::iter(single_url.into_iter().cycle().take(27))
                        .map(|url| {
                                let client = client.clone();
                                let arc_rate_limiter = arc_rate_limiter.clone();
                                async move {
                                        arc_rate_limiter.until_ready().await;
                                        client.get(url).send().await
                                }
                        })
                        .buffer_unordered(BURST_LIMIT.get() as usize);

                let mut count = 0;
                while let Some(result) = regulated_request_stream.next().await {
                        println!("{}: {:.2}", count, start_time.elapsed().as_secs_f64());
                        count += 1;
                        debug!(?result);
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

/// Example async function with retries
async fn _fetch_with_retry(client: &reqwest::Client, url: &str, max_retries: u32) -> Result<reqwest::Response> {
        let mut retries = 0;
        loop {
                match client.get(url).send().await {
                        Ok(response) => return Ok(response),
                        Err(e) if retries < max_retries => {
                                warn!("Request failed, retrying: {}", e);
                                retries += 1;
                                tokio::time::sleep(Duration::from_millis(2u64.pow(retries))).await;
                        }
                        Err(e) => {
                                error!("Request tried {} times without success. Last error returned: {}", retries, e);
                                return Err(e.into());
                        }
                }
        }
}

// #[cfg(target_arch = "wasm32")]
// fn main() {}
