//! Code with Vars Example
// ///////////////////////////////// -use- ///////////////////////////////// //
use indoc::indoc;

// ///////////////////////////////// -main- ///////////////////////////////// //
fn main() {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native("CodeEditor", native_options, Box::new(|_cc| Ok(Box::new(CodeVarsExample::default()))))
                .unwrap();
}
// ///////////////////////////////// -App Memory- ///////////////////////////////// //
//                                     and init
#[derive(Debug)]
pub struct CodeVarsExample {
        name: String,
        age: u32,
}
impl Default for CodeVarsExample {
        fn default() -> Self {
                Self { name: "Arthur".to_owned(), age: 42 }
        }
}

impl CodeVarsExample {
        /// egui-component: code samples in a grid
        fn samples_in_grid(&mut self, grid_name: &str, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
                // Note: we keep the code narrow so that the example fits on a mobile screen.

                egui::Grid::new(grid_name).striped(true).num_columns(2).show(ui, |ui| {
                        let Self { name, age } = self; // for brevity later on

                        rust_view_ui(ui, r#"ui.heading("Example");"#);
                        ui.heading("Example");
                        ui.end_row();

                        rust_view_ui(
                                ui,
                                indoc! { r#"
                        ui.horizontal(|ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(name);
                        });"#},
                        );
                        // Putting things on the same line using ui.horizontal:
                        ui.horizontal(|ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(name);
                        });
                        ui.end_row();

                        rust_view_ui(
                                ui,
                                indoc! { r#"
                        ui.add(
                                egui::DragValue::new(age)
                                .range(0..=120)
                                .suffix(" years"),
                        );"#},
                        );
                        ui.add(egui::DragValue::new(age).range(0..=120).suffix(" years"));
                        ui.end_row();

                        rust_view_ui(
                                ui,
                                indoc! {r#"
                        if ui.button("Increment").clicked() {
                                *age += 1;
                        }"#},
                        );
                        if ui.button("Increment").clicked() {
                                *age += 1;
                        }
                        ui.end_row();

                        rust_view_ui(ui, r#"ui.label(format!("{name} is {age}"));"#);
                        ui.label(format!("{name} is {age}"));
                        ui.end_row();
                })
        }

        fn code(&mut self, ui: &mut egui::Ui) {
                rust_view_ui(
                        ui,
                        indoc! {r"
                        pub struct CodeExample {
                                name: String,
                                age: u32,
                        }

                        impl CodeExample {
                            fn ui(&mut self, ui: &mut egui::Ui) {
                                // Saves us from writing `&mut self.name` etc
                                let Self { name, age } = self;"},
                );
                {
                        ui.horizontal(|ui| {
                                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                                let indentation = 2.0 * 4.0 * ui.fonts(|f| f.glyph_width(&font_id, ' '));
                                ui.add_space(indentation);

                                self.samples_in_grid("code", ui);
                        });
                }
                rust_view_ui(ui, "    }\n}");
        }
}

impl eframe::App for CodeVarsExample {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                egui::Window::new("My Window")
                        .min_width(375.0)
                        .default_size([390.0, 500.0])
                        .scroll(false)
                        .resizable([true, false]) // resizable so we can shrink if the text edit grows
                        .show(ctx, |ui| {
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

                                let mut theme =
                                        egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                                ui.collapsing("Theme", |ui| {
                                        theme.ui(ui);
                                        theme.store_in_memory(ui.ctx());
                                });

                                ui.separator();
                        });
        }
}

/// View some Rust code with syntax highlighting and selection.
fn rust_view_ui(ui: &mut egui::Ui, code: &str) {
        let language = "rs";
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        egui_extras::syntax_highlighting::code_view_ui(ui, &theme, code, language);
}
