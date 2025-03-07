//! Library

mod app;
mod error;

pub use app::WebCompatibleApp;
pub use error::{ErrKind, ErrWrapper, ToOther};

pub type WebCompatibleResult<T> = std::result::Result<T, ErrWrapper>;
