use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use std::any::type_name;

fn serialize_with_typename<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
        T: Serialize,
        S: Serializer,
{
        let mut map = serializer.serialize_map(Some(1))?;
        let key = type_name::<T>(); // Get the Rust type as a string
        map.serialize_entry(key, value)?;
        map.end()
}

#[derive(Serialize)]
struct Example {
        #[serde(serialize_with = "serialize_with_typename")]
        field: i32, // Can be any type
}

// ///////////////////////////////////////////////////////
fn main() {
        let example = Example { field: 42 };
        let json = serde_json::to_string_pretty(&example).unwrap();
        println!("{}", json);

        let pageinfo = PageInfo { page: 1, limit: 10, finished: false };
        let user1 = User { id: 1, name: "John Doe A".to_string() };
        let user2 = User { id: 2, name: "John Doe B".to_string() };
        let consumer1 = Consumer { id: 1, title: "Consumer Title A".to_string() };
        let consumer2 = Consumer { id: 2, title: "Consumer Title B".to_string() };

        let pageduser = PaginatedResponse::<User> { page: pageinfo.clone(), data: vec![user1, user2] };
        let pagedconsumer = PaginatedResponse::<Consumer> { page: pageinfo, data: vec![consumer1, consumer2] };

        let json = serde_json::to_string_pretty(&pageduser).unwrap();
        println!("{}", json);

        let json = serde_json::to_string_pretty(&pagedconsumer).unwrap();
        println!("{}", json);
}
// ///////////////////////////////////////////////////////

/// Approahc one (issues with Serde labelling)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
struct PaginatedResponse<T>
where
        T: Serialize + Deserialize<'static>,
{
        #[serde(flatten)]
        page: PageInfo,
        #[serde(flatten)]
        #[serde(serialize_with = "serialize_with_typename")]
        data: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct PageInfo {
        page: u32,
        limit: u8,
        finished: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct User {
        id: u32,
        name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Consumer {
        id: u32,
        title: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Resource {
        id: u32,
}
