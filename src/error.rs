//! Error & Result type

use std::io;

use derive_more::{Display, Error};
use tracing::{instrument, subscriber::SetGlobalDefaultError};

#[derive(Debug, Display, derive_more::From, Error)]
pub enum ErrorExample {
    // `custom` errors //

    // `packed` errors //
    #[display("CLI parsing library error: {}", source)]
    Clap { source: clap::Error },

    #[display("Error with tracing_subscriber::EnvFilter parsing env directive: {}",
              source)]
    EnvError {
        source: tracing_subscriber::filter::FromEnvError,
    },

    #[display("io error: {}", source)]
    Io { source: io::Error },

    #[display("parse error: {}", source)]
    ParseInt { source: std::num::ParseIntError },

    #[display("reqwest error: {}", source)]
    Reqwest { source: reqwest::Error },

    #[display("Error setting tracing subscriber default: {}", source)]
    TracingSubscriber { source: SetGlobalDefaultError },

    #[display("url parse error: {}", source)]
    Url { source: url::ParseError },

    // `other` errors //
    #[from(ignore)] // use `make_dyn_error` instead; would conflict with auto-derives
    #[display("Uncategorized Error (dyn error object): {}", source)]
    OtherDynError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[display(r#"Uncategorized string err: "{}""#, source_string)]
    OtherStringError { source_string: String },
}
impl ErrorExample {
    /// Convenience asscfunction for transforming an error into a compabtible *dyn error*.
    ///
    /// ```ignore
    /// use support::ErrKind;
    /// let clip = arboard::Clipboard::new().map_err(ErrKind::into_dyn_error)?;
    /// ```
    #[instrument(skip_all)]
    pub fn into_dyn_error<E>(error: E) -> Self
        where E: Into<Box<dyn std::error::Error + Send + Sync>> {
        Self::OtherDynError { source: error.into(), }
    }
}
