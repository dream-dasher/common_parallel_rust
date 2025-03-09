//! A simple code editor widget.

pub struct CodeEditor {
        language: String,
        code: String,
}

impl Default for CodeEditor {
        fn default() -> Self {
                Self {
                        language: "rs".into(),
                        code: "// A very simple example\n\
fn main() {\n\
\tprintln!(\"Hello world!\");\n\
}\n\
"
                        .into(),
                }
        }
}

fn main() {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native("simple code editor", native_options, Box::new(|_cc| Ok(Box::new(CodeEditor::default()))))
                .unwrap();
}

impl eframe::App for CodeEditor {
        #[expect(unused)]
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
                egui::CentralPanel::default().show(ctx, |ui| self.ui(ui));
        }
}
impl CodeEditor {
        fn ui(&mut self, ui: &mut egui::Ui) {
                let Self { language, code } = self;

                if cfg!(feature = "syntect") {
                        ui.horizontal(|ui| {
                                ui.label("Language:");
                                ui.text_edit_singleline(language);
                        });
                        ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label("Syntax highlighting powered by ");
                                ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
                                ui.label(".");
                        });
                } else {
                        ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label("Compile the demo with the ");
                                ui.code("syntax_highlighting");
                                ui.label(" feature to enable more accurate syntax highlighting using ");
                                ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
                                ui.label(".");
                        });
                }

                let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                ui.collapsing("Theme", |ui| {
                        ui.group(|ui| {
                                theme.ui(ui);
                                theme.clone().store_in_memory(ui.ctx());
                        });
                });

                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                ui.ctx(),
                                ui.style(),
                                &theme,
                                string,
                                language,
                        );
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(code)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter));
                });
        }
}
