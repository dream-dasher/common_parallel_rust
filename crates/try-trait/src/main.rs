/*!
# Playing with Traits
*/
// //////////////////////////////////// -use- //////////////////////////////////// //
use derive_more as dm;
use tracing::{self as tea, level_filters::LevelFilter};
use utilities::activate_global_default_tracing_subscriber;
// //////////////////////////////////// -main- //////////////////////////////////// //
fn main() -> Result<(), Box<dyn std::error::Error>> {
        let _writer_guard = activate_global_default_tracing_subscriber()
                .default_logging_level(LevelFilter::DEBUG)
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

        // rest-like
        let mut def_descriptions = Descriptions::make(None)?;
        def_descriptions.add_description((
                "Look, we made something via blanket trait.".to_string(),
                DescriptionKind::default(),
        ));
        def_descriptions.talkabout();
        Ok(())
}
// //////////////////////////////////// -function- //////////////////////////////////// //
// //////////////////////////////////// -traits rest-like- //////////////////////////////////// //
/// Deletable of class X returning Y
///
/// e.g.
/// ```ignore
/// let u_del_confirm = User::delete("user_id").await?;
/// let r_del_confirm = Record::delete("record_id").await?;
/// let p_del_confirm = Page::delete("page_id").await?;
/// ```
#[trait_variant::make(LocalDeletable: Send)]
trait _LocalDeletable {
        type DeleteId: std::fmt::Debug;
        type DeleteReturn: std::fmt::Debug;
        /// Instances should often have a retrievable Id
        fn _delete_id(&self) -> Option<Self::DeleteId>;
        /// endpoint to be added to base url for delete call
        fn _endpoint() -> &'static str {
                "api.path.getme--unimplemented"
        }
        /// delete call
        /// the use of `#[trait_variant::make(LocalDeletable: Send)]`
        /// generates `LocalDeletable` & `Deletable, the former has no `Send` bound, the latter does.
        fn _delete(id: Self::DeleteId) -> Result<Option<Self::DeleteReturn>, Box<dyn std::error::Error>>;
        // {
        //         let endpoint = Self::endpoint();
        //         tea::info!(?endpoint, ?id);
        //         Err("Not implemented".into())
        // }
}
/// Gettable of class X returning Y
///
/// e.g.
/// ```ignore
/// let users  = User::get::<Vec<User>>(None).await?;     // List Users
/// let user   = User::get::<User>("user_id").await?;     // Get User
/// let u_role = User::get::<UserRole>("user_id").await?; // Get User Role
/// ```
pub trait Gettable<T>
where
        T: Send,
{
        type GetId: std::fmt::Debug + Send;
        type Error: std::fmt::Debug + Send;
        type Client: Send;
        /// Default get_id is `None` and is unused by implementation
        fn get_id(&self) -> Option<Self::GetId> {
                None
        }
        /// endpoint to be added to base url for get call
        fn endpoint() -> &'static str {
                "api.path.getme--unimplemented"
        }
        /// get call
        ///fn get(id: Self::GetId, client: &Self::Client) -> Result<T, Self::Error>| Future ∈ Send
        fn get(id: Self::GetId, client: &Self::Client) -> impl Future<Output = Result<T, Self::Error>> + Send;
}
trait Makeable {
        type MakeId: std::fmt::Debug;
        type WhatsMade: std::fmt::Debug; // we're not using it here, but could!
        /// Default make is silly - takes 'thing' as id and
        fn make(id: Option<Self::MakeId>) -> Result<Self::WhatsMade, Box<dyn std::error::Error>>
        where
                Self::WhatsMade: Default,
        {
                if id.is_none() {
                        tea::info!(?id);
                        Ok(Self::WhatsMade::default())
                } else {
                        tea::info!(?id);
                        Err("Not implemented".into())
                }
        }
}
impl<T> Makeable for T
where
        T: Sized + Default + std::fmt::Debug,
{
        type MakeId = T;
        type WhatsMade = T;
}

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
/// Convert a value to a type 'T'.
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
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::FromStr, dm::From, dm::Into, dm::Display)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::From, dm::Into, dm::Display)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, dm::Display)]
enum DescriptionKind {
        #[default]
        Good,
        Bad,
}
