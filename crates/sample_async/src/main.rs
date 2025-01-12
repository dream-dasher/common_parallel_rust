//! Exploring some tokio init code.
//!
//! ## Note
//! **tokio** is not compatible with wasm target.

#[cfg(target_arch = "wasm32")]
fn main() {}
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
        // Needs:
        // - trait `IntoUrl`
        //   - pre-impl for `String` and `&str`
        const URL_DEFAULT: &str = "https://httpbin.org";

        // Basic
        let client = reqwest::Client::new();

        let res = client
                .post("http://httpbin.org/post")
                .body("the exact body that is sent")
                .send()
                .await?;

        println!("Response status {}", res.status());
        Ok(())
}
