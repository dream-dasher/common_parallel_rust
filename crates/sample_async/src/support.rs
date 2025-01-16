//! Support code

pub mod error;
mod hidden_value;
mod subscriber;

pub use error::ErrWrapper;
pub use hidden_value::{HiddenValue, HiddenValueError};
pub use subscriber::activate_global_default_tracing_subscriber;

pub type SampleResult<T> = std::result::Result<T, ErrWrapper>;
