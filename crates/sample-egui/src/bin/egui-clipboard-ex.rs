//! Near-direct copy from egui examples, modulo some specific setup based on their codebase
//!
//! ## Process of Making egui main examples *runnable*
//! The egui main examples are all components of a single app.
//! they have a trait `View` for which `self.ui(ui)` is defined.
//! That core `.ui(ui)` function just needs its code run within something that generates a UI.

// ///////////////////////////////// [ use ] ///////////////////////////////// //
use eframe::egui;

// ///////////////////////////////// [ main ] ///////////////////////////////// //
fn main() {
       let native_options = eframe::NativeOptions::default();
       eframe::run_native("Clipboard Test", native_options, Box::new(|cc| Ok(Box::new(ClipboardTest::new(cc))))).unwrap();
}

// ///////////////////////////////// [ App Memory ] ///////////////////////////////// //
//                                     and init
pub struct ClipboardTest {
       text: String,
}
impl Default for ClipboardTest {
       fn default() -> Self { Self { text: "Example text you can copy-and-paste".to_owned() } }
}
impl ClipboardTest {
       #[expect(unused)]
       fn new(cc: &eframe::CreationContext<'_>) -> Self { Self::default() }

       fn ui(&mut self, ui: &mut egui::Ui) {
              ui.label("egui integrates with the system clipboard.");
              ui.label("Try copy-cut-pasting text in the text edit below.");

              let text_edit_response = ui
                     .horizontal(|ui| {
                            let text_edit_response = ui.text_edit_singleline(&mut self.text);
                            if ui.button("📋").clicked() {
                                   ui.ctx().copy_text(self.text.clone());
                            }
                            text_edit_response
                     })
                     .inner;

              if !cfg!(target_arch = "wasm32") {
                     // These commands are not yet implemented on web
                     ui.horizontal(|ui| {
                            for (name, cmd) in [
                                   ("Copy", egui::ViewportCommand::RequestCopy),
                                   ("Cut", egui::ViewportCommand::RequestCut),
                                   ("Paste", egui::ViewportCommand::RequestPaste),
                            ] {
                                   if ui.button(name).clicked() {
                                          // Next frame we should get a copy/cut/paste-event…
                                          ui.ctx().send_viewport_cmd(cmd);

                                          // …that should en up here:
                                          text_edit_response.request_focus();
                                   }
                            }
                     });
              }
       }
}

// impl crate::Demo for ClipboardTest {
//         fn name(&self) -> &'static str {
//                 "Clipboard Test"
//         }

//         fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
//                 egui::Window::new(self.name()).open(open).show(ctx, |ui| {
//                         use crate::View as _;
//                         self.ui(ui);
//                 });
//         }
// }

// ///////////////////////////////// [ Core Loop ] ///////////////////////////////// //
impl eframe::App for ClipboardTest {
       #[expect(unused)]
       fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { egui::CentralPanel::default().show(ctx, |ui| self.ui(ui)); }
}
