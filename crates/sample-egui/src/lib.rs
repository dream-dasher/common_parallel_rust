//! Utility code for other Workspace Crates

mod app;
mod error;

pub use app::SampleApp;
pub use error::{ErrKind, ErrWrapper, ToOther};

pub type SampleResult<T> = std::result::Result<T, ErrWrapper>;
