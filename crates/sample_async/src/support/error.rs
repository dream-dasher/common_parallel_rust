//! Error & Result type for Day07 of Advent of Code 2024.
//!
//! ## Common ErrorKinds
//! // //
//! // // `custom` errors
//! // #[from(ignore)] // manually generate; would conflict with `OtherStringError` auto-derive
//! // #[display("Error splitting on ':' : {}", source_input)]
//! // InputNoColon { source_input: String },
//! // #[from(ignore)]
//! // #[display("Error extracting lines from input: {}", source_input)]
//! // InputNoLines { source_input: String },
//! // #[from(ignore)]
//! // #[display("error parsing char: {}", uninterpretable_char)]
//! // ParseChar { uninterpretable_char: char },
//! // #[from(ignore)]
//! // #[display("parse error: {}", source)]
//! // ParseInt { source: num::ParseIntError },
//! // #[display("Unparsable character: {}", source_char)]
//! // ParseOther { source_char: char },
//! // //
//! // // `packed` errors
//! // #[display("CLI parsing library error: {}", source)]
//! // Clap { source: clap::Error },
//! // #[display("Error with tracing_subscriber::EnvFilter parsing env directive: {}", source)]
//! // EnvError { source: tracing_subscriber::filter::FromEnvError },
//! // #[display("eframe (egui) error: {}", source)]
//! // EFrame { source: eframe::Error },
//! // #[display("io error: {}", source)]
//! // Io { source: io::Error },
//! // #[display("reqwest error: {}", source)]
//! // Reqwest { source: reqwest::Error },
//! // #[display("Error setting tracing subscriber default: {}", source)]
//! // TracingSubscriber { source: SetGlobalDefaultError },
//! // #[display("url parse error: {}", source)]
//! // Url { source: url::ParseError },
//! // //
//! // // `other` errors
//! // #[from(ignore)] // use `make_dyn_error` instead; would conflict with auto-derives
//! // #[display("Uncategorized Error (dyn error object): {}", source)]
//! // OtherDynError { source: Box<dyn std::error::Error + Send + Sync> },
//! // #[display(r#"Uncategorized string err: "{}""#, source_string)]
//! // OtherStringError { source_string: String },
//!
//! ## Utility reference
//! For adding backtrace to errors:
//! `#![feature(error_generic_member_access)]`
//! `use std::backtrace;`

use std::io;

use derive_more::{Display, Error};
use tracing::{instrument, subscriber::SetGlobalDefaultError};

// use derive_more::{Display, Error, derive::From};
#[derive(Debug, Display, derive_more::From, Error)]
pub enum ErrKind {
        // `custom` errors //

        // `packed` errors //
        #[display("CLI parsing library error: {}", source)]
        Clap { source: clap::Error },

        #[display("Error with tracing_subscriber::EnvFilter parsing env directive: {}", source)]
        EnvError { source: tracing_subscriber::filter::FromEnvError },

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
        OtherDynError { source: Box<dyn std::error::Error + Send + Sync> },

        #[display(r#"Uncategorized string err: "{}""#, source_string)]
        OtherStringError { source_string: String },
}
impl ErrKind {
        #[instrument(skip_all)]
        pub fn make_dyn_error<E>(error: E) -> Self
        where
                E: Into<Box<dyn std::error::Error + Send + Sync>>,
        {
                Self::OtherDynError { source: error.into() }
        }
}

#[derive(Display, Error)]
#[display(
        "error: {:#}\n\n\nspantrace capture: {:?}\n\n\nspantrace: {:#}",
        source,
        spantrace.status(),
        spantrace,
)]
pub struct ErrWrapper {
        source:    ErrKind,
        spantrace: tracing_error::SpanTrace,
        // backtrace: backtrace::Backtrace,
}
// Using custom display as debug so we can get SpanTrace auto printed.
impl std::fmt::Debug for ErrWrapper {
        #[instrument(skip_all)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self)
        }
}
impl<E> From<E> for ErrWrapper
where
        E: Into<ErrKind>,
{
        #[instrument(skip_all)]
        fn from(error: E) -> Self {
                Self {
                        source:    error.into(),
                        spantrace: tracing_error::SpanTrace::capture(),
                        // backtrace: backtrace::Backtrace::capture(),
                }
        }
}

pub trait ToOther {
        #[expect(dead_code)]
        fn to_other(self) -> ErrWrapper;
}
impl<E> ToOther for E
where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
        #[instrument(skip_all)]
        fn to_other(self) -> ErrWrapper {
                ErrKind::OtherDynError { source: self.into() }.into()
        }
}
