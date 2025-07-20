//! Primary body of egui code
//!

// ///////////////////////////////// [ use ] ///////////////////////////////// //

// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init

use egui::TextWrapMode;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SampleApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    some_bool: bool,
}
impl Default for SampleApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            some_bool: false,
        }
    }
}
impl SampleApp {
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
}

// ///////////////////////////////// [ Core Loop ] ///////////////////////////////// //
impl eframe::App for SampleApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

            // Miscellaneous tips and tricks

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0; // remove spacing between widgets
                // `radio_value` also works for enums, integers, and more.
                ui.radio_value(&mut self.some_bool, false, "Off");
                ui.radio_value(&mut self.some_bool, true, "On");
            });

            ui.group(|ui| {
                ui.label("Within a frame");
                ui.set_min_height(200.0);
                ui.push_id(1, |ui| {
                    ui.collapsing("Same header", |_ui| {}); // this is fine!
                });
                ui.push_id(2, |ui| {
                    ui.collapsing("Same header", |_ui| {}); // this is fine!
                });
                ui.push_id(3, |ui| {
                    ui.collapsing("Same header", |_ui| {}); // this is fine!
                });
            });

            ui.indent(1111, |ui| {
                ui.label("intendented section");
            });
            // A `scope` creates a temporary [`Ui`] in which you can change settings:
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                ui.group(|ui| {
                    ui.label("Within a frame2");
                    ui.set_min_height(200.0);
                });
                ui.indent(1121, |ui| {
                    ui.label("intendented section");
                });
                ui.label("This text will be red, monospace, and won't wrap to a new line");
            }); // the temporary settings are reverted here
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Egui Xp");

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
                "https://github.com/dream-dasher/common_parallel_rust/tree/workspace_init/",
                "Source code."
            ));

            // info lines at bottom of panel
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

// ///////////////////////////////// [ Elements ] ///////////////////////////////// //
