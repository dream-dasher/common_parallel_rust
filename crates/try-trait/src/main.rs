/*!
# Playing with Traits
*/

mod error;
use crate::error::ErrWrapper;
pub type SampleResult<T> = std::result::Result<T, ErrWrapper>;

use derive_more as dm;
use tracing::{self as tea};
use utilities::activate_global_default_tracing_subscriber;

fn main() -> SampleResult<()> {
        let _writer_guard = activate_global_default_tracing_subscriber()
                .maybe_default_logging_level(None)
                .maybe_error_logging_level(None)
                .call()?;
        tea::info!("Starting main");

        Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::FromStr, dm::From, dm::Into, dm::Display)]
struct NewString(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::From, dm::Into, dm::Display)]
struct Newu32(u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Descriptions {
        num_good: u32,
        num_bad: u32,
        descriptions: Vec<(String, DescriptionKind)>,
}
impl Descriptions {
        fn new() -> Self {
                Descriptions { num_good: 0, num_bad: 0, descriptions: Vec::new() }
        }
        fn add_description(&mut self, (description, kind): (String, DescriptionKind)) {
                match kind {
                        DescriptionKind::Good => self.num_good += 1,
                        DescriptionKind::Bad => self.num_bad += 1,
                }
                self.descriptions.push((description, kind));
        }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::Display)]
enum DescriptionKind {
        Good,
        Bad,
}
