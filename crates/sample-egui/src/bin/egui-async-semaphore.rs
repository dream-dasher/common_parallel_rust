/*!
# Egui + Async + Rate & Task Control

## Main motivation
How *much* do request-tasks cost.  A lot of API surface encourages making N tasks and then progressively completing them -- with limitations via buffering or semaphore or (with some more work) rate-limiting.
However, each Future takes memory.  Holding many Futures that won't be used until others are gone could (and does) eat said memory.
e.g. - queueing up **100_000 or 1_000_000** tasks in a JoinSet moves this app to using about **1/2 or 1 GB**, respectively.

The main alternatives are some sort of manual stream/iteration structure -- which is supported by tokio's async-stream crate and the `stream!` macro.  ...but requires code inside a macro context, where it's denied analytics support.
Another available pattern is an "actor" model with some generator task that rate-limits the generation of tasks.
We've used the later here (and contrasted it with raw generation).

`JoinSet` has conveniences, but requires sharing a `&mut` reference to add tasks.  Which necessitates a mutex or shenanigans.
`TaskTracker` works with only `&`, but requires additional code for Cancellation and for Result receipt.

## Note (on variety for its own sake)
This intentionally employs too many and sometimes awkwardly conflicting elements.  e.g. channels to transmit results while also polling a JoinSet, but not using the same results.
Or utilizing a mix of JoinSet and TaskTracker without leaning on the strengths of either.
It's intentionally variatals box to play with some options.
*/
// ///////////////////////////////// [ use ] ///////////////////////////////// //
use std::{error::Error,
          num::NonZeroU32,
          sync::{Arc, Mutex as StdMutex,
                 atomic::{AtomicUsize, Ordering},
                 mpsc::{Receiver as BlockingReceiver, Sender as BlockingSender}},
          time::Duration};

use eframe::egui;
use reqwest::{self, Client as ReqClient, Method, StatusCode, Url,
              header::{self, HeaderMap}};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::{sync::Semaphore, task::JoinSet, time::interval};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{Instrument, debug_span, info, instrument, trace};
use utilities::activate_global_default_tracing_subscriber;
// ///////////////////////////////// [ main ] ///////////////////////////////// //
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _writer_guard: tracing_appender::non_blocking::WorkerGuard =
        activate_global_default_tracing_subscriber().maybe_default_logging_level(None)
                                                    .maybe_error_logging_level(None)
                                                    .call()?;
    eframe::run_native("Async Future Memory Management",
                       eframe::NativeOptions::default(),
                       Box::new(|_cc| Ok(Box::new(FuturesApp::default()))))?;
    Ok(())
}
// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init
type JoinSetArcMutex<T> = Arc<StdMutex<JoinSet<T>>>;
#[derive(Debug)]
struct FuturesApp {
    // channel
    tx:                      BlockingSender<reqwest::StatusCode>,
    rx:                      BlockingReceiver<reqwest::StatusCode>,
    //  general
    _loading:                bool,
    client:                  ReqClient,
    semaphore:               Arc<Semaphore>,
    delay_sec:               u8,
    requests_to_queue:       NonZeroU32,
    // raw tasks
    join_set:                JoinSet<Result<(), Box<dyn Error + Send + Sync>>>,
    join_set_caged:          JoinSetArcMutex<Result<(), Box<dyn Error + Send + Sync>>>,
    // task generator
    task_gen_tracker:        TaskTracker,
    request_period:          Duration,
    request_tasks_to_create: Arc<AtomicUsize>,
    generator_cancel:        CancellationToken,
    // results
    count_200:               usize,
    count_400:               usize,
    count_other:             usize,
}
impl Default for FuturesApp {
    #[instrument]
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let semaphore = Arc::new(Semaphore::new(10));
        let client = generate_client().unwrap();
        let join_set = tokio::task::JoinSet::new();
        let join_set_caged = Arc::new(StdMutex::new(tokio::task::JoinSet::new()));
        let task_gen_tracker = TaskTracker::new();
        Self { // channel
               tx,
               rx,
               // general
               _loading: false,
               client,
               semaphore,
               delay_sec: 1,
               requests_to_queue: NonZeroU32::new(1).unwrap(),
               // raw tasks
               join_set,
               join_set_caged,
               // task generator
               task_gen_tracker,
               request_period: Duration::from_millis(100),
               request_tasks_to_create: Arc::new(AtomicUsize::new(0)),
               generator_cancel: CancellationToken::new(),
               // results
               count_200: 0,
               count_400: 0,
               count_other: 0 }
    }
}
impl FuturesApp {
    #[instrument]
    /// Generate a task that executes a semaphore-gated request.
    /// This *ought* to take self, but that would make calling it from an independent task (the generator generator) more difficult.
    fn queue_request(delay: u8,
                     client: ReqClient,
                     semaphore: Arc<Semaphore>,
                     tx: BlockingSender<StatusCode>,
                     join_set: &mut JoinSet<Result<(), Box<dyn Error + Send + Sync>>>,
                     ctx: egui::Context) {
        let endpoint = {
            const HTTPBIN_DELAY_URL: &str = "https://httpbin.org/delay";
            Url::parse(&format!("{}/{}", HTTPBIN_DELAY_URL, delay)).unwrap()
        };
        // NOTE: we're not actually using JoinSet -- in fact we're leaking due to lack of poll
        join_set.spawn(async move {
                           let req = client.request(Method::GET, endpoint)
                                           .build()
                                           .expect("should be valid reqwest");
                           info!(?req);
                           let _permit = semaphore.acquire().await;
                           ctx.request_repaint();
                           info!(?_permit);
                           let resp = client.execute(req).await?;
                           info!(?resp);
                           tx.send(resp.status())?;
                           // REPAINT
                           ctx.request_repaint();
                           Ok::<(), Box<dyn Error + Send + Sync>>(())
                       }.instrument(debug_span!("reqwest", ?delay)));
    }

    /// Generate a Generator of tasks that execute semaphore-gated requests.
    #[instrument]
    fn metered_queue_request(&mut self, ctx: egui::Context) {
        // ....................... [ cheap-clones-for-task ] ....................... //

        let requests_to_queue = self.requests_to_queue;
        let endpoint_delay = self.delay_sec;
        let semaphore = self.semaphore.clone();
        let tx = self.tx.clone();
        let mut interval = interval(self.request_period);
        let client = self.client.clone();
        let arc_mutex = self.join_set_caged.clone();
        let atomic_counter = self.request_tasks_to_create.clone();
        let cancel_token = self.generator_cancel.clone();
        // ....................... [ update-atomic-counter ] ....................... //
        atomic_counter.fetch_add(requests_to_queue.get() as usize, Ordering::Relaxed);
        // ----------------------------- [ spawn-semaphore-gated-reqwest-task ] ----------------------------- //
        self.task_gen_tracker
            .spawn(async move {
                       // let mut interval = interval(request_period);
                       for i in 0..requests_to_queue.get() {
                           tokio::select! {
                                   _ = interval.tick() => {
                                           trace!(i, "tick");
                                           let mut join_set_caged = arc_mutex.lock().unwrap();
                                           FuturesApp::queue_request(
                                                   endpoint_delay,
                                                   client.clone(),
                                                   semaphore.clone(),
                                                   tx.clone(),
                                                   &mut join_set_caged,
                                                   ctx.clone(),
                                           );
                                           drop(join_set_caged);
                                           atomic_counter.fetch_sub(1, Ordering::Relaxed);
                                   }
                                   _ = cancel_token.cancelled() => {
                                           let remaining_dropped = (requests_to_queue.get() - i) as usize;
                                           info!(iteration = i, remaining_dropped, "cancelled");
                                           atomic_counter.fetch_sub(remaining_dropped, Ordering::Relaxed);
                                           return;
                                   }
                           }
                           ctx.request_repaint();
                       }
                   }.instrument(debug_span!("metered request spawner", ?endpoint_delay)));
    }
}
// ///////////////////////////////// [ app accessory ] ///////////////////////////////// //
/// Struct to pull typicode responses into
/// Example of using 'typed' JSON with Serde
#[derive(Debug, Serialize, Deserialize)]
struct RemoteDelayResponse {
    data:    String,
    headers: JsonValue,
    url:     String,
    #[serde(flatten)]
    other:   JsonValue,
}
fn generate_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let default_headers = {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(header::USER_AGENT, "rust-reqwest-client".parse().unwrap());
        headers
    };
    let client = reqwest::Client::builder().https_only(true) // this will error for `http` (WARN: not compile-time checked)
                                           .use_rustls_tls()
                                           .default_headers(default_headers)
                                           .timeout(Duration::from_secs(30)) // default is *no* timeout
                                           .build()?;
    Ok(client)
}
// ///////////////////////////////// [ loop ] ///////////////////////////////// //
const NON_ZERO_MIN: NonZeroU32 = NonZeroU32::new(1).unwrap();
const NON_ZERO_MAX: NonZeroU32 = NonZeroU32::new(u32::MAX).unwrap();
impl eframe::App for FuturesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ check-'n-count ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        // NOTE: We would normally loop over the JoinSet, but we're intentionally over-using tools for exploration purposes.
        if let Ok(status) = self.rx.try_recv() {
            match status {
                _ if status.is_success() => self.count_200 += 1,
                _ if status.is_client_error() => self.count_400 += 1,
                _ => self.count_other += 1,
            }
            let _log = self.join_set.try_join_next();
            trace!(join_set_clear_result=?_log);
            let _log_mutexed = self.join_set_caged.lock().unwrap().try_join_next();
            trace!(join_set_clear_result_mutexed=?_log_mutexed);
            ctx.request_repaint();
        }
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ right-control-pane ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
                        ui.heading("Immediate Future Generation");
                        if ui.button(format!("Queue {} Request(s)", self.requests_to_queue))
                                .clicked()
                        {
                                info!("Queueing requests");
                                for _ in 1..=self.requests_to_queue.get() {
                                        FuturesApp::queue_request(
                                                self.delay_sec,
                                                self.client.clone(),
                                                self.semaphore.clone(),
                                                self.tx.clone(),
                                                &mut self.join_set,
                                                ctx.clone(),
                                        );
                                }
                                ctx.request_repaint();
                        }
                ui.label("If we used JoinSet instead of TaskTracker we could drop the generators without an explicit cancellation tooken.");
                        if ui.button("Drop Requests").clicked() {
                                info!("Aborting requests");
                                self.join_set.abort_all();
                                while self.join_set.try_join_next().is_some() {
                                        trace!("Clearing finished/aborted task from JoinSet")
                                }
                                ctx.request_repaint();
                        }
                });
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ left-control-pane ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Metered Future Generation");
            let mut request_period_ms = self.request_period.as_millis() as u64;
            if ui.add(egui::Slider::new(&mut request_period_ms, 10..=10000).logarithmic(true)
                                                                           .text("Request Period (ms)"))
                 .changed()
            {
                self.request_period = Duration::from_millis(request_period_ms);
            }
            if ui.button(format!("Queue {} Request(s) every {} ms",
                                 self.requests_to_queue,
                                 self.request_period.as_millis()))
                 .clicked()
            {
                info!("Queueing metered requests");
                self.metered_queue_request(ctx.clone());
                ctx.request_repaint();
            }
            if ui.button("Drop Requests").clicked() {
                info!("Aborting requests");
                if let Ok(mut join_set_caged) = self.join_set_caged.lock() {
                    join_set_caged.abort_all();
                    while join_set_caged.try_join_next().is_some() {
                        trace!("Clearing finished/aborted task from JoinSet")
                    }
                    ctx.request_repaint();
                }
            }
            if ui.button("Drop Request Generator(s)").clicked() {
                info!("Cancelling request generators");
                self.generator_cancel.cancel();
                // the cancellation state will persist (and appears irreversible): we provide the app with a new canellation token; (safe?)
                self.generator_cancel = CancellationToken::new();
                // note: no need to drain the generator queue, because it uses `TaskTracker` not `JoinSet`
                ctx.request_repaint();
            }
        });
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ [ display-pane ] ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Data Display");
            ui.label(format!("200 count: {}", self.count_200));
            ui.label(format!("400 count: {}", self.count_400));
            ui.label(format!("Other count: {}", self.count_other));
            ui.add_space(10.0);
            ui.label(format!("Semaphore available: {}", self.semaphore.available_permits()));
            ui.label(format!("Queued Requests: {}", self.join_set.len()));
            ui.label(format!("Queued Requests metered: {}", self.join_set_caged.lock().unwrap().len()));
            ui.label(format!("Request-Task Generators active: {}", self.task_gen_tracker.len()));
            ui.label(format!("Request-Tasks to Create: {}",
                             self.request_tasks_to_create.load(Ordering::Relaxed)));
            ui.add(egui::Slider::new(&mut self.delay_sec, 0..=10).text("Server Response Delay (sec)"));
            ui.add(egui::Slider::new(&mut self.requests_to_queue, NON_ZERO_MIN..=NON_ZERO_MAX)
                            .logarithmic(true)
                            .text("Number of requests to queue"));
        });
    }
}
