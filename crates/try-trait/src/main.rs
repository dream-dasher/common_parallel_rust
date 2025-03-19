/*!
# Playing with Traits
*/
use derive_more as dm;
use tracing::{self as tea};
use utilities::activate_global_default_tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
        let _writer_guard = activate_global_default_tracing_subscriber()
                .maybe_default_logging_level(None)
                .maybe_error_logging_level(None)
                .call()?;

        let mut descriptions = Descriptions::new();
        descriptions.add_description(("Good description".to_string(), DescriptionKind::Good));
        descriptions.add_description(("Bad description".to_string(), DescriptionKind::Bad));
        descriptions.add_description(("Bad2 description".to_string(), DescriptionKind::Bad));
        descriptions.add_description(("Bad3 description".to_string(), DescriptionKind::Bad));
        tea::debug!(?descriptions);
        let removed = descriptions.remove_description(4);
        tea::debug!(?removed);
        let removed = descriptions.remove_description(3);
        tea::debug!(?removed);
        tea::debug!(?descriptions);

        let out = descriptions.talkabout();
        tea::info!(?out);

        let nstring = NewString("Hello, world!".to_string());
        let nnum = Newu32(42);
        tea::debug!(?nstring);
        tea::debug!(?nnum);

        let nsin = nstring.get_innie();
        let nnin = nnum.get_innie();
        tea::debug!(nsin, nnin);
        Ok(())
}
// //////////////////////////////////// -traits- //////////////////////////////////// //
trait Infoable {
        fn talkabout(&self) -> String
        where
                Self: std::fmt::Debug,
        {
                tea::info!(?self);
                format!("{:?}", self)
        }
}

trait GotsAnInnie {
        type Innie;
        fn get_innie(&self) -> Self::Innie;
}

// //////////////////////////////////// -example structs to use- //////////////////////////////////// //
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::FromStr, dm::From, dm::Into, dm::Display)]
struct NewString(String);
impl GotsAnInnie for NewString {
        type Innie = String;
        fn get_innie(&self) -> Self::Innie {
                self.0.clone()
        }
}
impl Infoable for NewString {
        fn talkabout(&self) -> String {
                tea::info!(?self, "NewString");
                format!("{:?}", self)
        }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::From, dm::Into, dm::Display)]
struct Newu32(u32);
impl GotsAnInnie for Newu32 {
        type Innie = u32;
        fn get_innie(&self) -> Self::Innie {
                self.0
        }
}
impl Infoable for Newu32 {
        fn talkabout(&self) -> String {
                let innie = self.get_innie();
                tea::info!(?innie);
                format!("innie: {:?}", innie)
        }
}

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
        fn remove_description(&mut self, index: usize) -> Option<(String, DescriptionKind)> {
                if index >= self.descriptions.len() {
                        return None;
                }
                let (description, kind) = self.descriptions.remove(index);
                match kind {
                        DescriptionKind::Good => self.num_good -= 1,
                        DescriptionKind::Bad => self.num_bad -= 1,
                }
                Some((description, kind))
        }
}
impl Infoable for Descriptions {}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::Display)]
enum DescriptionKind {
        Good,
        Bad,
}
