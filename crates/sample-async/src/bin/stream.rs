// ///////////////////////////////////////// [ use ] ///////////////////////////////////////// //
// use tokio_stream::{self, StreamExt};
// use futures::{StreamExt, Stream, stream};

use async_stream::stream;
use futures::StreamExt;
use futures::pin_mut;

use std::error::Error;
// ///////////////////////////////////////// [ main ] ///////////////////////////////////////// //
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ classic ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        let mut stream = tokio_stream::iter(&[0, 1, 2]);

        while let Some(value) = stream.next().await {
                println!("Got {}", value);
        }
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ stream!-macro ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        let s = stream! {
            for i in 0..3 {
                yield i;
            }
        };
        pin_mut!(s); // needed for iteration

        while let Some(value) = s.next().await {
                println!("got {}", value);
        }
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ scratch ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        // ----------------------------- [ reqwest ] ----------------------------- //

        let client = reqwest::Client::builder().build()?;
        let resp = client.get("https://httpbin.org/get").send().await?;
        println!("headers: {:#?}", resp.headers());
        println!("url:     {}", resp.url());
        println!("status:  {:>99}", resp.status());
        println!("body:    {}", resp.text().await?);
        // ----------------------------- [ delayed ] ----------------------------- //
        let mut interval = time::interval(Duration::from_millis(1000));
        interval.reset(); // <-- this is critical to get started (or other options)
        // `interval.tick().await;` would also work; as first tick completes immediately
        let delstream = stream! {
            for i in 0..30 {
                let now = time::Instant::now();
                tokio::time::sleep(Duration::from_millis(rand::random_range(0..1000))).await;
                let elapsed_2 = now.elapsed();

                let now = time::Instant::now();
                interval.tick().await;
                let elapsed_1 = now.elapsed();

                yield (i, elapsed_1.as_millis(), elapsed_2.as_millis(), elapsed_1.as_millis() + elapsed_2.as_millis());
            }
        };
        pin_mut!(delstream); // needed for iteration

        while let Some((i, tick, sleep, total)) = delstream.next().await {
                println!("{}: tick: {}, sleep: {}, total: {}", i, tick, sleep, total);
        }
        Ok(())
}

use tokio::time::{self, Duration};

// #[tokio::main]
// async fn main() {
//         let mut interval = time::interval(Duration::from_millis(10));

//         interval.tick().await; // ticks immediately
//         interval.tick().await; // ticks after 10ms
//         interval.tick().await; // ticks after 10ms

//         // approximately 20ms have elapsed.
// }
