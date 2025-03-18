// ///////////////////////////////// -use- ///////////////////////////////// //
use eframe::egui;
use reqwest::{
        Client, Method, Url,
        header::{self, HeaderMap},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ///////////////////////////////// -main- ///////////////////////////////// //
fn main() -> Result<(), Box<dyn std::error::Error>> {
        let async_runtime = tokio::runtime::Runtime::new().expect("Tokio runtime should be creatable.");
        let _enter = async_runtime.enter();

        // loop to keep thread up (?)
        std::thread::spawn(move || {
                async_runtime.block_on(async {
                        loop {
                                tokio::time::sleep(Duration::from_secs(3600)).await;
                        }
                })
        });

        eframe::run_native(
                "Hello egui + tokio",
                eframe::NativeOptions::default(),
                Box::new(|_cc| Ok(Box::new(ChannelApp::default()))),
        )?;

        Ok(())
}
// ///////////////////////////////// -App Memory- ///////////////////////////////// //
//                                     and init
struct ChannelApp {
        // Sender/Receiver for async notifications.
        tx: std::sync::mpsc::Sender<Vec<Todo>>,
        rx: std::sync::mpsc::Receiver<Vec<Todo>>,

        // jsons: Vec<serde_json::Value>,
        todos: Vec<Todo>,

        loading: bool,
        error: Option<String>,

        _selected_todo: Option<usize>,
}
impl Default for ChannelApp {
        fn default() -> Self {
                let (tx, rx) = std::sync::mpsc::channel();
                Self { tx, rx, todos: Vec::new(), loading: false, error: None, _selected_todo: None }
        }
}

/// Struct to pull typicode responses into
/// Example of using 'typed' JSON with Serde
#[derive(Debug, Serialize, Deserialize)]
struct Todo {
        #[serde(rename = "userId")]
        user_id: i32,
        id: i32,
        title: String,
        completed: bool,
}

// ///////////////////////////////// -loop- ///////////////////////////////// //
impl eframe::App for ChannelApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                if let Ok(todos) = self.rx.try_recv() {
                        self.todos = todos;
                        self.loading = false;
                        ctx.request_repaint();
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Async Fetch Example - grabs TODOs from Typicode");
                        ui.label("Press the button to initiate an HTTP request.");

                        if self.loading {
                                ui.spinner();
                                ui.label("Loading...");
                        } else if let Some(err) = &self.error {
                                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                        } else if self.todos.is_empty() {
                                ui.label("No todos to display");
                        } else {
                                ui.label(format!("Loaded {} todos", self.todos.len()));
                        }

                        ui.add_space(10.0);

                        for i in 1..=4 {
                                if ui.button(format!("Request id: {}", i)).clicked() && !self.loading {
                                        self.loading = true;
                                        let client = generate_client().unwrap();
                                        send_request(client, i, self.tx.clone(), ctx.clone());
                                }
                        }

                        // ui.horizontal(|ui| {
                        //         for user_id in 1..=5 {
                        //                 if ui.button(format!("User {}", user_id)).clicked() && !self.loading {
                        //                         self.fetch_todos(Some(user_id), ctx.clone());
                        //                 }
                        //         }
                        // });
                });
                egui::SidePanel::right("panel").show(ctx, |ui| {
                        ui.heading("Todos");

                        if self.todos.is_empty() {
                                ui.label("No todos to display");
                        } else {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                        for todo in &self.todos {
                                                ui.horizontal(|ui| {
                                                        ui.checkbox(&mut todo.completed.clone(), "");
                                                        let text = format!("{} - {}", todo.id, todo.title);
                                                        ui.label(text);
                                                });
                                        }
                                });
                        }
                });
        }
}
fn send_request(client: Client, req_id: u8, tx: std::sync::mpsc::Sender<Vec<Todo>>, ctx: egui::Context) {
        const URL_TYPICODE: &str = "https://jsonplaceholder.typicode.com";
        let todos_typicode = Url::parse(URL_TYPICODE).unwrap().join("/todos").unwrap();
        tokio::spawn(async move {
                let response = client
                        .request(Method::GET, todos_typicode)
                        .query(&[("userId", req_id)])
                        // .query(&[("userId", "1"), ("completed", "false")])
                        .send()
                        .await
                        .unwrap();
                let todos: Vec<Todo> = response.json().await.unwrap();
                let _ = tx.send(todos);
                ctx.request_repaint();
        });
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

// let mut suspense = EguiSuspense::reloadable(|cb| {
//         std::thread::spawn(move || {
//                 std::thread::sleep(std::time::Duration::from_secs(1));
//                 cb(if rand::random() { Ok("Hello".to_string()) } else { Err("OOPSIE WOOPSIE!".to_string()) });
//         });
// });

// eframe::run_simple_native("DnD Simple Example", Default::default(), move |ctx, _frame| {
//         CentralPanel::default().show(ctx, |ui| {
//                 // This will show a spinner while loading and an error message with a
//                 // retry button if the callback returns an error.
//                 suspense.ui(ui, |ui, data, state| {
//                         ui.label(format!("Data: {:?}", data));

//                         if ui.button("Reload").clicked() {
//                                 state.reload();
//                         }
//                 });
//         });
// })?;
