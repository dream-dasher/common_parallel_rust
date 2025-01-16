//! Utility code for other Workspace Crates

mod app;
mod support;

pub use app::TemplateApp;
pub use support::{SampleResult, activate_global_default_tracing_subscriber, *};
