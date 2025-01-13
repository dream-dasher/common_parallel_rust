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
use support::{Result, activate_global_default_tracing_subscriber};
use tracing::{Level as L, event as tea};

// #[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
        let _writer_guard = activate_global_default_tracing_subscriber(None, None)?;

        // Needs:
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

        // header-module
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
        // xhs httpbin.org/headers accept:'application/json' Authorization:'prettyplease' Fanciful:'yesm'
        // `Client`
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

        // see RequestBuilder
        let request = client
                .request(Method::GET, base_httpbin.join("/headers")?)
                .header("Authorization", "prettyplease")
                .header("Fanciful", "ladeeda")
                .query(&[("query_key", "query_value")])
                .body("the exact body that is sent")
                .build()?;
        tea!(L::DEBUG, ?request);

        // Response:
        // - status
        //   - error_for_status
        // - cookies
        // - url
        // - text
        // - json
        let response = client.execute(request).await?;

        tea!(L::DEBUG, ?response);
        tea!(L::INFO, resp_status=?response.status());
        Ok(())
}

// #[cfg(target_arch = "wasm32")]
// fn main() {}
