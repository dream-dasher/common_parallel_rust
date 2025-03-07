#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use sample_webapp::*;
// use utilities::activate_global_default_tracing_subscriber;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> WebCompatibleResult<()> {
        // #[cfg(debug_assertions)]
        use utilities::activate_global_default_tracing_subscriber;
        let _writer_guard: tracing_appender::non_blocking::WorkerGuard = activate_global_default_tracing_subscriber()
                .maybe_env_default_level(None)
                .maybe_trace_error_level(None)
                .call()?;

        let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                        .with_inner_size([400.0, 300.0])
                        .with_min_inner_size([300.0, 220.0]),
                // .with_icon(
                //         // NOTE: Adding an icon is optional
                //         eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                //                 .expect("Failed to load icon"),
                // ),
                ..Default::default()
        };
        eframe::run_native("Egui Xp", native_options, Box::new(|cc| Ok(Box::new(WebCompatibleApp::new(cc)))))?;
        Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() -> WebCompatibleResult<()> {
        use eframe::wasm_bindgen::JsCast as _;

        // Redirect `log` message to `console.log` and friends:
        // eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        // The following causes the webpage to never load (probably the writer...) todo!()
        // use utilities::activate_global_default_tracing_subscriber;
        // let _writer_guard: tracing_appender::non_blocking::WorkerGuard = activate_global_default_tracing_subscriber()
        //         .maybe_env_default_level(None)
        //         .maybe_trace_error_level(None)
        //         .call()?;

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
                let document = web_sys::window().expect("No window").document().expect("No document");

                let canvas = document
                        .get_element_by_id("the_canvas_id")
                        .expect("Failed to find the_canvas_id")
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .expect("the_canvas_id was not a HtmlCanvasElement");

                let start_result = eframe::WebRunner::new()
                        .start(canvas, web_options, Box::new(|cc| Ok(Box::new(WebCompatibleApp::new(cc)))))
                        .await;

                // Remove the loading text and spinner:
                if let Some(loading_text) = document.get_element_by_id("loading_text") {
                        match start_result {
                                Ok(_) => {
                                        loading_text.remove();
                                }
                                Err(e) => {
                                        loading_text.set_inner_html(
                                                "<p> The app has crashed. See the developer console for details. </p>",
                                        );
                                        panic!("Failed to start eframe: {e:?}");
                                }
                        }
                }
        });
        Ok(())
}
