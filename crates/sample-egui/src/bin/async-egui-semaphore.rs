// ///////////////////////////////// [ use ] ///////////////////////////////// //
use eframe::egui;
use reqwest::Client as ReqClient;
use reqwest::{self, Request};
use reqwest::{
        Method, Url,
        header::{self, HeaderMap},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::error::Error;
use std::sync::{Arc, mpsc as blocking_mpsc};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{info, info_span, instrument};
use utilities::activate_global_default_tracing_subscriber;
// ///////////////////////////////// [ main ] ///////////////////////////////// //
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//         // tokio::runtime::Builder::new_current_thread()
//         // ^-- stalls app; tasks never run due to thread block by event loop

//         // same as #[tokio::main(flavor = "multi_thread")], which is dfault for #[tokio::main]
//         tokio::runtime::Builder::new_multi_thread()
//                 .enable_time()
//                 .enable_io()
//                 .build()
//                 .expect("Tokio runtime should be creatable.")
//                 .block_on(async {
//                         eframe::run_native(
//                                 "Hello egui + tokio",
//                                 eframe::NativeOptions::default(),
//                                 Box::new(|_cc| Ok(Box::new(ChannelApp::default()))),
//                         )
//                         .unwrap();
//                 });

//         Ok(())
// }
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let _writer_guard: tracing_appender::non_blocking::WorkerGuard = activate_global_default_tracing_subscriber()
                .maybe_default_logging_level(None)
                .maybe_error_logging_level(None)
                .call()?;
        eframe::run_native(
                "Hello egui + tokio",
                eframe::NativeOptions::default(),
                Box::new(|_cc| Ok(Box::new(FuturesApp::default()))),
        )?;
        Ok(())
}
// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init
#[derive(Debug)]
struct FuturesApp {
        tx: blocking_mpsc::Sender<reqwest::StatusCode>,
        rx: blocking_mpsc::Receiver<reqwest::StatusCode>,
        _loading: bool,
        semaphore: Arc<Semaphore>,
        client: ReqClient,
        _reqwests: Vec<Request>,
        join_set: JoinSet<Result<(), Box<dyn Error + Send + Sync>>>,
        delay_sec: u8,
        count_200: usize,
        count_400: usize,
        count_other: usize,
}
impl Default for FuturesApp {
        #[instrument]
        fn default() -> Self {
                let (tx, rx) = std::sync::mpsc::channel();
                let semaphore = Arc::new(Semaphore::new(10));
                let client = generate_client().unwrap();
                let join_set = tokio::task::JoinSet::new();
                let _reqwests = Vec::new();
                Self {
                        tx,
                        rx,
                        _loading: false,
                        delay_sec: 1,
                        semaphore,
                        client,
                        _reqwests,
                        join_set,
                        count_200: 0,
                        count_400: 0,
                        count_other: 0,
                }
        }
}
impl FuturesApp {
        #[instrument]
        fn send_request(&mut self, delay: u8, ctx: egui::Context) {
                let endpoint = {
                        const HTTPBIN_DELAY_URL: &str = "https://httpbin.org/delay";
                        Url::parse(&format!("{}/{}", HTTPBIN_DELAY_URL, delay)).unwrap()
                };
                let client = self.client.clone();
                let tx = self.tx.clone();
                let semaphore = self.semaphore.clone();
                self.join_set.spawn(async move {
                        let span = info_span!("reqwest", delay);
                        let _guard = span.enter();
                        let req = client
                                .request(Method::GET, endpoint)
                                .build()
                                .expect("should be valid reqwest");
                        info!(?req);
                        let _permit = semaphore.acquire().await;
                        info!(?_permit);
                        let resp = client.execute(req).await?;
                        info!(?resp);
                        tx.send(resp.status())?;
                        // REPAINT
                        ctx.request_repaint();
                        Ok::<(), Box<dyn Error + Send + Sync>>(())
                });
        }
}
// ///////////////////////////////// [ app accessory ] ///////////////////////////////// //
/// Struct to pull typicode responses into
/// Example of using 'typed' JSON with Serde
#[derive(Debug, Serialize, Deserialize)]
struct RemoteDelayResponse {
        data: String,
        headers: JsonValue,
        url: String,
        #[serde(flatten)]
        other: JsonValue,
}
fn generate_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
        let default_headers = {
                let mut headers = HeaderMap::new();
                headers.insert(header::ACCEPT, "application/json".parse().unwrap());
                headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
                headers.insert(header::USER_AGENT, "rust-reqwest-client".parse().unwrap());
                headers
        };
        let client = reqwest::Client::builder()
                .https_only(true) // this will error for `http` (WARN: not compile-time checked)
                .use_rustls_tls()
                .default_headers(default_headers)
                .timeout(Duration::from_secs(30)) // default is *no* timeout
                .build()?;
        Ok(client)
}
// ///////////////////////////////// [ loop methods ] ///////////////////////////////// //
// ///////////////////////////////// [ loop ] ///////////////////////////////// //
impl eframe::App for FuturesApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                // check message and update count
                if let Ok(status) = self.rx.try_recv() {
                        match status {
                                _ if status.is_success() => self.count_200 += 1,
                                _ if status.is_client_error() => self.count_400 += 1,
                                _ => self.count_other += 1,
                        }
                        ctx.request_repaint();
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Async Fetch Example - grabs TODOs from Typicode");
                        ui.label(format!("200 count: {}", self.count_200));
                        ui.label(format!("400 count: {}", self.count_400));
                        ui.label(format!("Other count: {}", self.count_other));
                        ui.add_space(10.0);
                        ui.label(format!("Semaphore available: {}", self.semaphore.available_permits()));
                        // for i in 1..=4 {
                        //         if ui.button(format!("Request id: {}", i)).clicked() && !self.loading {
                        //                 self.loading = true;
                        //                 send_request(self.client.clone(), i, self.tx.clone(), ctx.clone());
                        //         }
                        // }
                        // ui.horizontal(|ui| {
                        //         for user_id in 1..=5 {
                        //                 if ui.button(format!("User {}", user_id)).clicked() && !self.loading {
                        //                         self.fetch_todos(Some(user_id), ctx.clone());
                        //                 }
                        //         }
                        // });
                });
                egui::SidePanel::right("right_panel").show(ctx, |ui| {
                        ui.heading("Right Panel");
                        ui.label("This is the right panel.");
                        ui.add(egui::Slider::new(&mut self.delay_sec, 0..=100));
                        if ui.button("Send Request").clicked() {
                                self.send_request(self.delay_sec, ctx.clone());
                        }
                });
        }
}
