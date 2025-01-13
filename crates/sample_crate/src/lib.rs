//! Utility code for other Workspace Crates

mod app;
mod support;

pub use app::TemplateApp;
pub use support::{Result, activate_global_default_tracing_subscriber};
