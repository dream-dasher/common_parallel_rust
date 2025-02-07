//! Utility code for other Workspace Crates

mod hidden_value;
mod subscriber;

pub use hidden_value::{HiddenValue, HiddenValueError};
pub use subscriber::activate_global_default_tracing_subscriber;
