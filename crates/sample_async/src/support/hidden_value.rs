//! Wrapper to prevent accidental logging of sensitive values.
//!
//! ## Design Note
//! This wrapper's primary purpose it to prevent accidental logging of sensitive values.
//! It additionally provide some methods to support common means of acquireing such values
//! (e.g. from environment) and means of providing custom debug values
//! for logging (e.g. it is common practice to log the last ~4 chars of an api key).
//!
//! This type does *not* attempt to provide memory security.  The `zeroize` crate it is
//! tempting, however it would seem to give a false sense of security if directly applied here.
//! Zeroization on the running of a destructor would not ensure that copies of the values weren't made
//! nor that the value wasn't moved without zeroization of the earlier value.  More broadly, as a property
//! of the physical implementation of the code it would not be reliably testable by common methods
//! -- and the implementation behavior could be changed by compiler optimizations, specific target, and
//! a variety of other factors.
//!
//! There may be some promise in in making a `Pin` version of HiddenValue and zeroizing on it's destruction.
//! However, even were that to offer the desired guarantees (and it would be non-trivial to determine) and we
//! would ensure that `HiddenValuePin` did not implement`Unpin` the use of `.expose()` would mean that the
//! sensitive value itself was not protected.
//!
//! **TLDR**: memory safety is interesting, but that is an express non-goal.  This is just to prevent logging or similar
//! textual leaks.
//!
//! ## Example
//! ```ignore
//! use std::{env, num::NonZeroUsize};
//!
//! use dotenvy::dotenv;
//! use hidden_value::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!         tracing_subscriber::fmt::init();
//!         const TEST_KEY: &str = "EXAMPLE_LOCAL_ENV_KEY_MESSAGE";
//!         // dotenv()?; // this will prefer existing env vars over .env file
//!         // let test_key = env::var(TEST_KEY)?;
//!         // println!("direct read of test_key from env: {}", test_key);
//!         let asv: HiddenValue<String> = HiddenValue::from_env_builder()
//!                 .key("TEST_KEY")
//!                 .load_env_file(true)
//!                 .reveal_len(NonZeroUsize::new(4).unwrap())
//!                 .build()?;
//!         println!("key:{}\n obfuscated val: {:?}", TEST_KEY, &asv);
//!         println!("key:{}\n exposed val: {}", TEST_KEY, &asv.expose_value());
//!         let hnum: HiddenValue<u32> = HiddenValue::builder().value(123_456_789).build()?;
//!         println!("hidden number: obfuscated: {:?}", &hnum);
//!         println!("hidden number: exposed: {}", &hnum.expose_value());
//!
//!         Ok(())
//! }
//! ```
use core::fmt;
use std::{env, ffi::OsStr, num::NonZeroUsize};

use bon::bon;
use derive_more::{Display, Error, From};
use dotenvy::dotenv;
use tracing::{self, debug, error, info, instrument, trace};

#[derive(Debug, Display, From, Error)]
pub enum HiddenValueError {
        #[display("Reveal length ({requested}) exceeds value's UTF-8 char length ({actual})")]
        RevealLengthTooLong { requested: usize, actual: usize },
        #[display("Env var not found: {}", source)]
        EnvVar { source: std::env::VarError },
        #[display("Dotenv error: {}", source)]
        Dotenv { source: dotenvy::Error },
}

/// Authorization credentials required for remote access
///
/// Note `rest_key` is "REST" in the sense of particular-CRUD flavored RPC
#[derive(Clone)]
pub struct HiddenValue<T> {
        value:      T,
        obf_string: Option<String>,
}
impl<T> fmt::Debug for HiddenValue<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.obf_string {
                        None => write!(f, "HiddenValue {{ REDACTED }}"),
                        Some(ref masked) => write!(f, r#"HiddenValue {{ REDACTED.."{}" }}"#, masked),
                }
        }
}
#[bon]
impl HiddenValue<std::string::String> {
        /// Attempt to find key in environment, optionally loading local or parent `.env` file first.
        /// All keys in `.env` file will be loaded if not present in environment. (Not just the presented key.)
        ///
        /// ## Internal Note
        /// I don't love the flow of this function.  I don't like loading an entire `.env` file for one key file for one key.
        /// And the error clarity on file vs environment precedence is lacking and similarly not nicely match by code flow.
        #[instrument(skip(key))]
        #[builder(start_fn = from_env_builder, finish_fn = build)]
        pub fn new_from_env<K>(
                /// Environment key to use to grab value to hide.
                /// The value will be read and stored as a UTF-8 string.
                key: K,
                /// Whether to first search for and load a `.env` file in local or parental directories.
                /// Will prefer current environment if a loaded value would conflict.
                load_env_file: bool,
                /// How many and whether to reveal the last n characters of value in debug representation.
                /// e.g. `reveal_len: Some(4)` would enable logging the last 4 value of an api-key.
                ///
                /// ## 'Fallible'
                /// This will error if the reveal length is not *strictly* *less* than the UTF-8 character length of the value.
                reveal_len: Option<NonZeroUsize>,
        ) -> Result<Self, HiddenValueError>
        where
                K: AsRef<OsStr>,
        {
                trace!(key_lossy=?key.as_ref().to_string_lossy());
                // maybe load .env to env
                if load_env_file {
                        match dotenv() {
                                Err(dotenv_err) => {
                                        info!(%dotenv_err, "No `.env` file found in local or parent directories..")
                                }
                                Ok(_) => tracing::debug!("Found and read .env file."),
                        };
                }
                // look for value in env
                let value = match env::var(&key) {
                        Err(env_err) => {
                                error!(%env_err, "Key not found in env.");
                                Err(env_err)?
                        }
                        Ok(value) => value,
                };
                // maybe generate masked value
                let masked_string: Option<String> = if let Some(reveal_len) = reveal_len {
                        let reveal_len: usize = reveal_len.get();
                        if value.len() <= reveal_len {
                                Err(HiddenValueError::RevealLengthTooLong {
                                        requested: reveal_len,
                                        actual:    value.len(),
                                })?
                        }
                        // last n chars (UTF-8)
                        Some(value.chars().skip(value.len() - reveal_len).collect())
                } else {
                        None
                };

                HiddenValue::builder()
                        .value(value)
                        .maybe_obf_string(masked_string)
                        .build()
        }
}
#[bon]
impl<T> HiddenValue<T> {
        /// Create a new HiddenValue instance.
        /// Optionally add an 'obfuscate string' to use as part of the debug representation of the wrapper.
        /// **WARN**: obf_string is meant to take and hold an obfuscated string.  It will hold and reveal whatever it is given.
        /// (Future changes may specialize this function and add checks for obfuscation.  Currently it is up to the caller to ensure.
        /// This function takes values with many or no direct routes to debug or string representations.)
        ///
        /// ## Fallibility: none
        /// This method returns a `Result` for future backward compatibility.
        /// In particular for the ability to specialize on types where 'obfuscated string' can be checked
        /// for possible non-obfuscation.
        /// Currently, however, it cannot fail.
        ///
        /// ## Internal Note
        /// `Into<Option<String>` vs `S: Into<String> .. Option<S>`
        /// The latter requires a type to be given for a None (e.g. `None::<String>`)
        /// The former does not.
        /// However, the former does not automatically trigger bon's optional methods.
        /// And default does not play nicely with the it either.
        /// (which can be remedied by way requesting into for `Option<String>`)
        /// Hence, in the absence of `bon` the following is preferred:
        /// ```ignore
        /// pub fn new(value: T, masked_string: impl Into<Option<String>>) -> Self {
        ///     let masked_string = masked_string.into();
        ///     ...
        /// ```
        #[builder]
        #[instrument(skip_all)]
        pub fn new(
                /// Value to hide. (From accidental logging, printing, etc.)
                value: T,
                /// Optional String to use as an obfuscating debug representation of the value.
                ///
                /// As the value may be of an aribitrary type the caller must do any transforms or obfuscations
                /// needed to create a value that can be coereced to a String for storage.
                #[builder(into)]
                obf_string: Option<String>,
        ) -> Result<Self, HiddenValueError> {
                if let Some(ref obf_string) = obf_string {
                        debug!(
                                ?obf_string,
                                "note: Due to generality of value types we cannot check that the 'obfuscated string' actually obfuscates."
                        );
                };
                Ok(Self { value, obf_string })
        }

        /// Expose the value of the key.
        ///
        /// ## Note
        /// This method is `must_use` both to clarify that it is not a side-effect based method
        /// and to keep uses cleanly.  While we are not explicitly protecting its presence in memory
        /// , nor even zeroizing on destruction (which doesn't ensure clean up in all locations it may have
        /// existed), keeping exposure intentional still appears to be best practice.
        #[must_use]
        #[instrument(skip_all)]
        pub fn expose_value(&self) -> &T {
                trace!("exposing hidden value");
                &self.value
        }
}

// Manual ('spot') testing.
#[cfg(test)]
mod tests {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use super::*;

        #[test]
        fn test_basic_hidden_value() {
                let secret = "my_secret_value".to_string();
                let hidden = HiddenValue::builder().value(secret.clone()).build().unwrap();

                assert_eq!(hidden.expose_value(), &secret);
                assert_eq!(format!("{:?}", hidden), "HiddenValue { REDACTED }");
        }

        #[test]
        fn test_partial_reveal() {
                const TEST_SECRET: &str = "1234567890";
                const TEST_OBF_STRING: &str = "7890";
                let secret = TEST_SECRET.to_string();
                let hidden = HiddenValue::builder()
                        .value(secret)
                        .obf_string(TEST_OBF_STRING)
                        .build()
                        .unwrap();
                assert_eq!(format!("{:?}", hidden), format!("HiddenValue {{ REDACTED..\"{}\" }}", TEST_OBF_STRING));
        }

        #[test]
        fn test_env_value() {
                const TEST_KEY: &str = "TEST_KEY";
                const TEST_VALUE: &str = "abcdefghi";
                let test_value_last_4 = &TEST_VALUE.chars().skip(TEST_VALUE.len() - 4).collect::<String>();
                // SAFETY: Test code only. Sets an env variable.
                //         Cost of collision should be low.
                //         (And test should be run in independent process.)
                #[expect(unsafe_code)]
                unsafe {
                        std::env::set_var(TEST_KEY, TEST_VALUE)
                };
                let hidden = HiddenValue::from_env_builder()
                        .key(TEST_KEY)
                        .load_env_file(false)
                        .reveal_len(NonZeroUsize::new(4).unwrap())
                        .build()
                        .unwrap();
                assert_eq!(hidden.expose_value(), TEST_VALUE);
                assert_eq!(format!("{:?}", hidden), format!("HiddenValue {{ REDACTED..\"{}\" }}", test_value_last_4));
        }

        #[test]
        fn test_reveal_length_too_long() {
                const TEST_KEY_2: &str = "TEST_KEY_2";
                const TEST_VALUE_2: &str = "ABCDEFGHI";
                // SAFETY: Test code only. Sets an env variable.
                //         Cost of collision should be low.
                //         (And test should be run in independent process.)
                #[expect(unsafe_code)]
                unsafe {
                        std::env::set_var(TEST_KEY_2, TEST_VALUE_2)
                };
                let result = HiddenValue::from_env_builder()
                        .key(TEST_KEY_2)
                        .load_env_file(false)
                        .reveal_len(NonZeroUsize::new(20).unwrap())
                        .build();

                assert!(matches!(result, Err(HiddenValueError::RevealLengthTooLong { .. })));
        }
}

// QuickCheck tests
#[cfg(test)]
mod quickcheck_tests {
        use quickcheck_macros::quickcheck;

        use super::*;

        #[quickcheck]
        fn qc_test_hidden_value_preserves_content(value: String) -> bool {
                let hidden = HiddenValue::builder().value(value.clone()).build().unwrap();
                hidden.expose_value() == &value
        }

        #[quickcheck]
        fn qc_test_reveal_length_validation(value_len: u16, reveal_len: Option<NonZeroUsize>) -> bool {
                const TEST_KEY_QC: &str = "TEST_KEY_QC";
                let value = "x".repeat(value_len as usize);
                // SAFETY: Test code only. Sets an env variable.
                //         Cost of collision should be low.
                //         (And test should be run in independent process.)
                #[expect(unsafe_code)]
                unsafe {
                        std::env::set_var(TEST_KEY_QC, value)
                };
                match reveal_len {
                        Some(reveal_len) => {
                                // let reveal_len_usize = reveal_len.get();
                                let hidden = HiddenValue::from_env_builder()
                                        .key(TEST_KEY_QC)
                                        .load_env_file(false)
                                        .reveal_len(reveal_len)
                                        .build();
                                if reveal_len.get() >= value_len as usize { hidden.is_err() } else { hidden.is_ok() }
                        }
                        None => HiddenValue::from_env_builder()
                                .key(TEST_KEY_QC)
                                .load_env_file(false)
                                .build()
                                .is_ok(),
                }
        }
}

#[cfg(test)]
mod insta_tests {

        use super::*;

        #[test]
        fn insta_test_string_hidden_value_debug() {
                const TEST_VALUE_STR: &str = "alphabetagaga";
                let hidden_str = HiddenValue::builder().value(TEST_VALUE_STR).build().unwrap();
                insta::assert_debug_snapshot!(hidden_str, @"HiddenValue { REDACTED }");

                const TEST_VALUE_STR_GREEK: &str = "αβγαγα";
                let hidden_str_greek = HiddenValue::builder().value(TEST_VALUE_STR_GREEK).build().unwrap();
                insta::assert_debug_snapshot!(hidden_str_greek, @"HiddenValue { REDACTED }");

                const TEST_VALUE_NUM: i32 = -910_050_019;
                let hidden_num = HiddenValue::builder().value(TEST_VALUE_NUM).build().unwrap();
                insta::assert_debug_snapshot!(hidden_num, @"HiddenValue { REDACTED }");
        }

        #[test]
        fn insta_test_string_hidden_value_with_obfuscation_debug() {
                const TEST_VALUE_STR: &str = "alphabetagaga";
                const TEST_OBF_STR: &str = "gaga";
                let hidden_str = HiddenValue::builder()
                        .value(TEST_VALUE_STR)
                        .obf_string(TEST_OBF_STR)
                        .build()
                        .unwrap();
                insta::assert_debug_snapshot!(hidden_str, @r#"HiddenValue { REDACTED.."gaga" }"#);

                const TEST_VALUE_STR_GREEK: &str = "αβγαγα";
                const TEST_OBF_STR_GREEK: &str = "γα";
                let hidden_str_greek = HiddenValue::builder()
                        .value(TEST_VALUE_STR_GREEK)
                        .obf_string(TEST_OBF_STR_GREEK)
                        .build()
                        .unwrap();
                insta::assert_debug_snapshot!(hidden_str_greek, @r#"HiddenValue { REDACTED.."γα" }"#);

                const TEST_VALUE_NUM: i32 = -910_050_019;
                const TEST_OBF_NUM: &str = "0_019";
                let hidden_num = HiddenValue::builder()
                        .value(TEST_VALUE_NUM)
                        .obf_string(TEST_OBF_NUM)
                        .build()
                        .unwrap();
                insta::assert_debug_snapshot!(hidden_num, @r#"HiddenValue { REDACTED.."0_019" }"#);
        }

        #[test]
        fn insta_test_hidden_value_result() {
                let hidden_result = HiddenValue::builder().value(12345).build();
                insta::assert_debug_snapshot!(hidden_result, @r"
                Ok(
                    HiddenValue { REDACTED },
                )
                ");
        }
}
