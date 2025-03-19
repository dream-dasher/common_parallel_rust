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

        // no traits; just using impls for a multi-field type with internal logic (Bad & Good values match num such values in Vec)
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

        // Using Infoable with pre-implemented trait
        let out = descriptions.talkabout();
        tea::info!(?out);

        // no traits; just raw newtype constructions
        let nstring = NewString("Hello, world!".to_string());
        let nnum = Newu32(42);
        tea::debug!(?nstring);
        tea::debug!(?nnum);

        // Using GotsAnInnie - simple use of ass-type
        let nsin = nstring.get_innie();
        let nnin = nnum.get_innie();
        tea::debug!(nsin, nnin);

        // Using the ToNumOfType method -- blanket impl with trait & impl bounded
        let intou32: u32 = nnum.0.to_num();
        let intou64: u64 = nnum.0.to_num();
        let intoi64: i64 = nnum.0.to_num();
        let intof64: f64 = nnum.0.to_num();
        tea::debug!(intou32, intou64, intoi64, intof64, ?nnum);

        Ok(())
}
// //////////////////////////////////// -function- //////////////////////////////////// //
// //////////////////////////////////// -traits- //////////////////////////////////// //
/// trait with default implementation
trait Infoable {
        fn talkabout(&self) -> String
        where
                Self: std::fmt::Debug,
        {
                tea::info!(?self);
                format!("{:?}", self)
        }
}
/// trait with type parameter
trait GotsAnInnie {
        type Innie;
        fn get_innie(&self) -> Self::Innie;
}
trait ToNumOfType<T>
where
        Self: Copy + Into<T>,
{
        fn to_num(&self) -> T {
                (*self).into()
        }
}
// //////////////////////////////////// -trait generic impl- //////////////////////////////////// //
impl<T, U> ToNumOfType<T> for U where U: Copy + Into<T> {}
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
