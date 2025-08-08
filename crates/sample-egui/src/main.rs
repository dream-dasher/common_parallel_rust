use sample_egui::*;
use utilities::activate_global_default_tracing_subscriber;

fn main() -> SampleResult<()> {
        // #[cfg(debug_assertions)]
        let _writer_guard: tracing_appender::non_blocking::WorkerGuard =
                activate_global_default_tracing_subscriber()
                        .maybe_default_logging_level(None)
                        .maybe_error_logging_level(None)
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
        eframe::run_native(
                "Egui Xp",
                native_options,
                Box::new(|cc| Ok(Box::new(SampleApp::new(cc)))),
        )?;
        Ok(())
}
