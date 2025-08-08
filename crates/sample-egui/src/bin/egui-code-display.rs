//! Code with Vars Example

// ///////////////////////////////// [ use ] ///////////////////////////////// //
use indoc::indoc;

// ///////////////////////////////// [ main ] ///////////////////////////////// //
fn main() {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
                "CodeEditor",
                native_options,
                Box::new(|_cc| Ok(Box::new(CodeVarsExample::default()))),
        )
        .unwrap();
}
// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init
#[derive(Debug)]
pub struct CodeVarsExample {
        name:      String,
        age:       u32,
        l_val:     f32,
        r_val:     f32,
        _show_bar: bool,
}
impl Default for CodeVarsExample {
        fn default() -> Self {
                Self {
                        name:      "Arthur".to_owned(),
                        age:       42,
                        l_val:     40.0,
                        r_val:     20.0,
                        _show_bar: true,
                }
        }
}
impl CodeVarsExample {
        /// egui-component: code samples in a grid
        fn samples_in_grid(
                &mut self,
                grid_name: &str,
                ui: &mut egui::Ui,
        ) -> egui::InnerResponse<()> {
                // Note: we keep the code narrow so that the example fits on a mobile screen.

                egui::Grid::new(grid_name)
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                                // let Self { name, age,  } = self; // for brevity later on

                                rust_view_ui(ui, r#"ui.heading("Example");"#);
                                ui.heading("Example");
                                ui.end_row();
                                // ----------------

                                rust_view_ui(ui, indoc! { r#"
                        ui.horizontal(|ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(name);
                        });"#});
                                // Putting things on the same line using ui.horizontal:
                                ui.horizontal(|ui| {
                                        ui.label("Name");
                                        ui.text_edit_singleline(&mut self.name);
                                });
                                ui.end_row();
                                // ----------------

                                rust_view_ui(ui, indoc! { r#"
                        ui.add(
                                egui::DragValue::new(&mut self.age)
                                .range(0..=120)
                                .suffix(" years"),
                        );"#});
                                ui.add(egui::DragValue::new(&mut self.age)
                                        .range(0..=120)
                                        .suffix(" years"));
                                ui.end_row();
                                // ----------------

                                rust_view_ui(ui, indoc! {r#"
                        if ui.button("Increment").clicked() {
                                self.age += 1;
                        }"#});
                                if ui.button("Increment")
                                        .clicked()
                                {
                                        self.age += 1;
                                }
                                ui.end_row();
                                // ----------------

                                rust_view_ui(
                                        ui,
                                        r#"ui.label(format!("{} is {}", self.name, self.age));"#,
                                );
                                ui.label(format!("{} is {}", self.name, self.age));
                                ui.end_row();
                                // ----------------
                        })
        }

        fn code(&mut self, ui: &mut egui::Ui) {
                rust_view_ui(ui, indoc! {r"
                        pub struct CodeExample {
                                name: String,
                                age: u32,
                        }

                        impl CodeExample {
                            fn ui(&mut self, ui: &mut egui::Ui) {
                                // Saves us from writing `&mut self.name` etc
                                let Self { name, age } = self;"});
                {
                        ui.horizontal(|ui| {
                                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                                let indentation =
                                        2.0 * 4.0 * ui.fonts(|f| f.glyph_width(&font_id, ' '));
                                ui.add_space(indentation);

                                self.samples_in_grid("code", ui);
                        });
                }
                rust_view_ui(ui, "    }\n}");
        }
}

impl eframe::App for CodeVarsExample {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                egui::SidePanel::right("my_panel").show(ctx, |ui| {
                        // // Warning about odd behavior
                        // ui.scope(|ui| {
                        //         ui.style_mut().visuals.override_text_color = Some(egui::Color32::from_rgb(255, 100, 0));
                        //         ui.label("⚠ Warning: Progress bar shows odd behavior when adjusting");
                        //         ui.label("left scroll position or changing l_size values.");
                        // });
                        ui.add_sized([self.l_val, self.r_val], egui::Label::new("Hello World!"));
                        ui.add_sized(
                                [self.l_val, self.r_val],
                                egui::DragValue::new(&mut self.l_val)
                                        .range(5.0..=f32::MAX)
                                        .prefix("x"),
                        );
                        ui.add_sized(
                                [self.l_val, self.r_val],
                                egui::DragValue::new(&mut self.r_val)
                                        .range(5.0..=f32::MAX)
                                        .prefix("y"),
                        );
                        // // Add a button to toggle progress bar visibility
                        // if ui.button(if self.show_bar { "Hide Progress Bar" } else { "Show Progress Bar" })
                        //         .clicked()
                        // {
                        //         self.show_bar = !self.show_bar;
                        // }

                        // // Only show the progress bar if show_bar is true
                        // if self.show_bar {
                        //         ui.add(ProgressBar::new(0.43));
                        // }
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                        ui.scope(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
                                self.code(ui);
                        });

                        ui.separator();
                        ui.separator();
                        ui.separator();

                        rust_view_ui(ui, &format!("{self:#?}"));

                        ui.separator();
                        ui.separator();

                        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
                                ui.ctx(),
                                ui.style(),
                        );
                        ui.collapsing("Theme", |ui| {
                                theme.ui(ui);
                                theme.store_in_memory(ui.ctx());
                        });

                        ui.separator();
                });
                egui::TopBottomPanel::bottom("bottom_panel")
                        .min_height(200.0)
                        .show(ctx, |ui| {
                                const BOOKOFSHADERS_LINK: &str = "https://thebookofshaders.com";
                                const ATOMNLOCKS_LINK: &str = "https://marabos.nl/atomics/";
                                ui.label("Bottom Panel");
                                if ui.button("open up web link: new tab")
                                        .clicked()
                                {
                                        ui.label("Button clicked!");
                                        ctx.open_url(egui::OpenUrl::new_tab(BOOKOFSHADERS_LINK));
                                }
                                if ui.button("open up web link: same tab")
                                        .clicked()
                                {
                                        ui.label("Button clicked!");
                                        ctx.open_url(egui::OpenUrl::same_tab(ATOMNLOCKS_LINK));
                                }
                        });
        }
}

/// View some Rust code with syntax highlighting and selection.
fn rust_view_ui(ui: &mut egui::Ui, code: &str) {
        let language = "rs";
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        egui_extras::syntax_highlighting::code_view_ui(ui, &theme, code, language);
}
