use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use std::any::type_name;

fn serialize_with_typename<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
        T: Serialize,
        S: Serializer,
{
        let mut map = serializer.serialize_map(Some(1))?;

        // Get the full type name
        let full_type_name = type_name::<T>();

        // Extract the simple type name
        let simple_type_name = extract_simple_type_name(full_type_name);

        // Pluralize it by adding 's'
        let pluralized = format!("{}s", simple_type_name.to_lowercase());

        map.serialize_entry(&pluralized, value)?;
        map.end()
}

// Helper function to extract the simple type name
fn extract_simple_type_name(full_name: &str) -> &str {
        // Handle Vec<T> case
        if let Some(start_idx) = full_name.find('<') {
                if let Some(end_idx) = full_name.rfind('>') {
                        let inner_part = &full_name[start_idx + 1..end_idx];

                        // Extract module path and type name
                        if let Some(last_colon) = inner_part.rfind("::") {
                                return &inner_part[last_colon + 2..]; // Skip the "::"
                        }
                        return inner_part;
                }
        }

        // Handle direct type case
        if let Some(last_colon) = full_name.rfind("::") {
                return &full_name[last_colon + 2..];
        }

        full_name
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
