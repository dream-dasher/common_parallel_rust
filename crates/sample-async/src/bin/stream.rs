// ///////////////////////////////////////// [ use ] ///////////////////////////////////////// //
// use tokio_stream::{self, StreamExt};
// use futures::{StreamExt, Stream, stream};
use async_stream::stream;
use futures::StreamExt;
use futures::pin_mut;
use owo_colors::OwoColorize as _;
use std::error::Error;
use tokio::time::{self, Duration};
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
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ timed-calls ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        let client = reqwest::Client::builder().build()?;
        let mut interval = time::interval(Duration::from_millis(500));
        // interval.reset(); // <-- critical to start // alt: `interval.tick().await`
        let mut now_yield = time::Instant::now();
        let delstream = stream! {
            for i in 0..30 {

                let now_req = time::Instant::now();
                let resp = client.get("https://httpbin.org/get").send().await.unwrap();
                let elapsed_req = now_req.elapsed();

                let now_tick = time::Instant::now();
                interval.tick().await;
                let elapsed_tick = now_tick.elapsed();

                let elapsed_yield = now_yield.elapsed();
                yield (i, elapsed_tick.as_millis(), elapsed_req.as_millis(), elapsed_yield.as_millis(), resp);
                now_yield = time::Instant::now();

            }
        };

        pin_mut!(delstream); // needed for iteration
        while let Some((i, tick, req, yielded, resp)) = delstream.next().await {
                println!(
                        "{:>4}: request: {:>4}  -  tick: {:>4}  -  yield: {:>4}  -  status: {:>4}",
                        i,
                        req.red(),
                        tick.cyan(),
                        yielded.purple(),
                        resp.status().green(),
                );
        }
        // ----------------------------- [ reqwest ] ----------------------------- //
        let client = reqwest::Client::builder().build()?;
        let resp = client.get("https://httpbin.org/get").send().await?;
        println!("headers: {:#?}", resp.headers());
        println!("url:     {}", resp.url());
        println!("status:  {:>99}", resp.status());
        println!("body:    {}", resp.text().await?);
        // ----------------------------- [ delayed ] ----------------------------- //
        let mut interval = time::interval(Duration::from_millis(1000));
        interval.reset(); // <-- critical to start // alt: `interval.tick().await`
        let mut now_yield = time::Instant::now();
        let delstream = stream! {
            for i in 0..30 {
                let now_sleep = time::Instant::now();
                time::sleep(Duration::from_millis(rand::random_range(0..1000))).await;
                let elapsed_sleep = now_sleep.elapsed();

                let now_tick = time::Instant::now();
                interval.tick().await;
                let elapsed_tick = now_tick.elapsed();

                let elapsed_yield = now_yield.elapsed();
                yield (i, elapsed_tick.as_millis(), elapsed_sleep.as_millis(), elapsed_yield.as_millis());
                now_yield = time::Instant::now();

            }
        };

        pin_mut!(delstream); // needed for iteration
        while let Some((i, tick, sleep, total)) = delstream.next().await {
                println!(
                        "{:>4}: sleep: {:>4}  -  tick: {:>4}  -  yield: {:>4}",
                        i,
                        sleep.cyan(),
                        tick.purple(),
                        total.blue()
                );
        }
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ end ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        Ok(())
}
