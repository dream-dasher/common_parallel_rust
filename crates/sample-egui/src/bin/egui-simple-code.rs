//! Code Editing Example

// ///////////////////////////////// [ use ] ///////////////////////////////// //
use indoc::indoc;

// ///////////////////////////////// [ main ] ///////////////////////////////// //
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "CodeEditor",
        native_options,
        Box::new(|_cc| Ok(Box::new(CodeEditorExample::default()))),
    )
    .unwrap();
}
// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CodeEditorExample {
    language: String,
    code: String,
}
impl Default for CodeEditorExample {
    /// Default to `.rs` (rust) syntax with starting code sample
    fn default() -> Self {
        Self {
            language: "rs".into(),
            code: CODE_SAMPLE.into(),
        }
    }
}

// ///////////////////////////////// [ Core Loop ] ///////////////////////////////// //
impl eframe::App for CodeEditorExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Self { language, code } = self;

            ui.horizontal(|ui| {
                ui.set_height(0.0);
                ui.label("An example of syntax highlighting in a TextEdit.");
                // ui.add(crate::egui_github_link_file!());
            });

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

            let mut code_theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    code_theme.ui(ui);
                    code_theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                    ui.ctx(),
                    ui.style(),
                    &code_theme,
                    string.as_str(),
                    language,
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}

// ///////////////////////////////// [ Reference Value ] ///////////////////////////////// //
const CODE_SAMPLE: &str = indoc! {r#"
impl eframe::App for CodeEditorExample {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                egui::CentralPanel::default().show(ctx, |ui| {
                        let Self { language, code } = self;
                        ui.horizontal(|ui| {
                                ui.set_height(0.0);
                                ui.label("An example of syntax highlighting in a TextEdit.");
                                // ui.add(crate::egui_github_link_file!());
                        });
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
                        let mut code_theme =
                                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                        ui.collapsing("Theme", |ui| {
                                ui.group(|ui| {
                                        code_theme.ui(ui);
                                        code_theme.clone().store_in_memory(ui.ctx());
                                });
                        });
                        let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                        ui.ctx(),
                                        ui.style(),
                                        &code_theme,
                                        string.as_str(),
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
                });
        }
}"#};
