//! Exploring some tokio init code.
//!
//! ## Note
//! **tokio** is not compatible with wasm target.

mod support;

use std::time::Duration;

use support::Result;
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
        use support::activate_global_default_tracing_subscriber;
        let _writer_guard = activate_global_default_tracing_subscriber(None, None)?;

        // Needs:
        // - trait `IntoUrl`
        //   - pre-impl for `String` and `&str`
        const URL_DEFAULT: &str = "https://httpbin.org";

        // client:
        // - `Arc` used internally
        let client = reqwest::Client::builder()
                .https_only(true) // this will error for `http` (WARN: not compile-time checked)
                .use_rustls_tls()
                // .default_headers(headers)
                .timeout(Duration::from_secs(30))
                .build()?;

        let request = client
                .post("https://httpbin.org/post")
                .body("the exact body that is sent")
                .build()?;

        println!("Method: {:?}", request.method());
        println!("URL: {:?}", request.url());
        println!("Headers: {:?}", request.headers());
        println!("Body: {:?}", request.body());

        let response = client.execute(request).await?;

        println!("Response status {}", response.status());
        Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
