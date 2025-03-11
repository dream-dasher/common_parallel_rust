//! Primary body of egui code
//!

// ///////////////////////////////// -use- ///////////////////////////////// //
use egui::{Key, ScrollArea};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

// ///////////////////////////////// -App Memory- ///////////////////////////////// //
//                                     and init

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WebCompatibleApp {
        // Example stuff:
        label: String,
        #[serde(skip)] // Don't serialize; note default used when serializing
        value: f32,
        #[serde(skip)]
        press_history: String,
        #[serde(skip)]
        thread_results: String,
        #[serde(skip)]
        rx: Option<Receiver<String>>,
}

impl Default for WebCompatibleApp {
        fn default() -> Self {
                Self {
                        // Example stuff:
                        label: "Hello World!".to_owned(),
                        value: 2.7,
                        press_history: String::new(),
                        thread_results: String::new(),
                        rx: None,
                }
        }
}

impl WebCompatibleApp {
        /// Called once before the first frame.
        pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
                // This is also where you can customize the look and feel of egui using
                // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

                // Load previous app state (if any).
                // Note that you must enable the `persistence` feature for this to work.
                if let Some(storage) = cc.storage {
                        return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
                }

                Default::default()
        }

        /// Receiver added to App
        /// WARN: ctx repaint request may be needed to ensure UI starts regular updating while waiting for data
        fn spawn_sleep_thread(&mut self, ms: u64) -> Result<(), String> {
                if self.rx.is_none() {
                        let (tx, rx) = mpsc::channel();
                        self.rx = Some(rx);
                        // ctx.request_repaint(); // ensure app knows to repaint (and initiate minimum repaint times as a result)
                        // ^ nope, we'd need to pass ctx -- will keep track of this in loop

                        thread::spawn(move || {
                                thread::sleep(Duration::from_millis(ms));
                                tx.send(format!("Thread sleep completed after {} ms", ms))
                                        .expect("receiver should always be present in main thread");
                        });
                        Ok(())
                } else {
                        Err("A thread is already running - please wait for it to complete".to_string())
                }
        }
}

// ///////////////////////////////// -Core Loop- ///////////////////////////////// //
impl eframe::App for WebCompatibleApp {
        /// Called by the frame work to save state before shutdown.
        fn save(&mut self, storage: &mut dyn eframe::Storage) {
                eframe::set_value(storage, eframe::APP_KEY, self);
        }

        /// Called each time the UI needs repainting, which may be many times per second.
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                        // The top panel is often a good place for a menu bar:

                        egui::menu::bar(ui, |ui| {
                                // NOTE: no File->Quit on web pages!
                                ui.menu_button("File", |ui| {
                                        if ui.button("Quit").clicked() {
                                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                        }
                                });
                                ui.add_space(16.0);

                                egui::widgets::global_theme_preference_buttons(ui);
                        });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                        // The central panel the region left after adding TopPanel's and SidePanel's
                        // ui.heading("WebAppCompatible Egui ∈ Eframe"); // < need a font that supports this
                        ui.heading("WebAppCompatible Egui in Eframe");

                        ui.horizontal(|ui| {
                                ui.label("Write something: ");
                                ui.text_edit_singleline(&mut self.label);
                        });

                        ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
                        if ui.button("Increment").clicked() {
                                self.value += 1.0;
                        }

                        ui.separator();

                        ui.add(egui::github_link_file!(
                                "https://github.com/dream-dasher/common_parallel_rust/",
                                "Source code."
                        ));

                        ui.separator();
                        const BOOKOFSHADERS_LINK: &str = "https://thebookofshaders.com";
                        const ATOMNLOCKS_LINK: &str = "https://marabos.nl/atomics/";
                        ui.label("links");
                        if ui.button("open up web link: new tab").clicked() {
                                ui.label("Button clicked!");
                                ctx.open_url(egui::OpenUrl::new_tab(BOOKOFSHADERS_LINK));
                        }
                        if ui.button("open up web link: same tab").clicked() {
                                ui.label("Button clicked!");
                                ctx.open_url(egui::OpenUrl::same_tab(ATOMNLOCKS_LINK));
                        }
                });

                egui::SidePanel::right("input panel").show(ctx, |ui| {
                        ui.heading("Pres/Hold/Release example. Press A to test.");
                        if ui.button("Clear").clicked() {
                                self.press_history.clear();
                        }
                        ScrollArea::vertical()
                                .auto_shrink(false)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                        ui.monospace(self.press_history.to_string());
                                });
                        if ctx.input(|i| i.key_pressed(Key::A)) {
                                self.press_history.push_str("\nPressed");
                                ui.ctx().request_repaint();
                        }
                        if ctx.input(|i| i.key_down(Key::A)) {
                                self.press_history.push_str("\nHeld");
                                ui.ctx().request_repaint();
                        }
                        if ctx.input(|i| i.key_released(Key::A)) {
                                self.press_history.push_str("\nReleased");
                                ui.ctx().request_repaint();
                        }
                });

                egui::TopBottomPanel::bottom("thread_panel").min_height(200.0).show(ctx, |ui| {
                        // Warning for WebAssembly environments
                        #[cfg(target_arch = "wasm32")]
                        {
                                ui.colored_label(
                                        egui::Color32::from_rgb(255, 80, 0),
                                        indoc::indoc!("⚠ WARNING: You appear to be running in WebAssembly.
                                        The thread-spawning commands below are *EXPECTED* to fail in this environment.  (Likely freezing the app and requiring a restart/reload.)
                                        They are here principally for demonstrating and exploring these differences.")
                                );
                                ui.separator();
                        }
                        // Note for non-WebAssembly environments
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                                ui.colored_label(
                                        egui::Color32::from_rgb(0,80,255),
                                        indoc::indoc!("ATTENTION: The following, thread-spawning commands, are target-sensitive.
                                        They *should* work in your environment, but are expected to fail in a web environment.
                                        They are here principally for demonstrating and exploring these differences.")
                                );
                                ui.separator();
                        }

                        // check channel
                        if let Some(rx) = &self.rx {
                                ui.colored_label(egui::Color32::GOLD, "⚡ Thread running");
                                if let Ok(message) = rx.try_recv() {
                                        self.thread_results.push_str(&format!("\n✔ {}", message));
                                        self.rx = None;
                                        ctx.request_repaint();
                                }
                                ctx.request_repaint_after(Duration::from_millis(100)); // ensure regular repaints if receiver present
                        } else {
                                ui.colored_label(egui::Color32::GRAY, "(no background thread running)");
                        }
                        ui.heading("Thread Example");
                        ui.horizontal(|ui| {
                                for i in [10, 100, 1_000, 10_000] {
                                        if ui.button(format!("Sleep {i}ms")).clicked() {
                                                match self.spawn_sleep_thread(i) {
                                                        Ok(_) => {
                                                                self.thread_results
                                                                        .push_str(&format!("\n-> Thread started.  Waiting {i}ms..."));
                                                        }
                                                        Err(_) => {
                                                                self.thread_results
                                                                        .push_str("\n xxx Failed start. We are awaiting the termination of the previous thread.");
                                                        }
                                                }
                                        }
                                }
                                if ui.button("Clear Results").clicked() {
                                        self.thread_results.clear();
                                        ctx.request_repaint();
                                }
                        });
                        // Thread result display
                        ScrollArea::vertical()
                                .auto_shrink(false)
                                .stick_to_bottom(true)
                                .max_height(100.0)
                                .show(ui, |ui| {
                                        ui.monospace(&self.thread_results);
                                });
                });
        }
}

/// Short, uncentered, horozontal line noting `egui` and `eframe` are both used and ofering hyperlinks to each.
fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
                ui.label(".");
        });
}
