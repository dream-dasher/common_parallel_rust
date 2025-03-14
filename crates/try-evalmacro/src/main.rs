//! Exploring some tokio init code.
//!
//! ## TLDR
//! - Reqwest
//!   - awkward http request crate
//!   - works with raw strings or a fragile/foot-gunning `Url` crate re-export
//!   - unfortunate necessity
//!     - `httpx` (python) or `ureq` (rust) are similar - string-like operation appears to be the norm
//! - Sqlx
//!   - straight SQL in your rust, but with compile time checking of of Rust types and that database supports the requests
//! - Tokio
//!
//! ## Accessories
//! - sites set up to allow http request testing/experimenting
//!   - [htpbin](https://httpbin.org)
//!   - [typicode: jsonplaceholder](https://jsonplaceholder.typicode.com)
//!
//! ## Note
//! **tokio** is not compatible with wasm target.

use eval_macro::eval;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json;

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

/// Approahc one (issues with Serde labelling)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct PaginatedResponse<T> {
        #[serde(flatten)]
        page: PageInfo,
        data: Vec<T>,
}

/// Approahc one (issues with Serde labelling)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PaginatedResponseAlt<T> {
        #[serde(flatten)]
        page: PageInfo,
        data: T,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Users(Vec<User>);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Consumers(Vec<Consumer>);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Resources(Vec<Resource>);

fn main() {
        eval! {
                let components = ["User", "Consumer", "Resource"];
                        for listable_resource in components.iter() {
                        output! {
                                #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
                                struct Paginated{{listable_resource}}Response {
                                        #[serde(flatten)]
                                        page: PageInfo,
                                        #[serde(rename = "{listable_resource}s")]
                                        data: Vec<{{listable_resource}}>,
                                }
                        }
                }
        }

        let dummy_pageinfo = PageInfo { page: 1, limit: 10, finished: false };
        let dummy_users = vec![User { id: 1, name: "A".to_string() }, User { id: 2, name: "B".to_string() }];
        let dummy_consumers =
                vec![Consumer { id: 1, title: "AA".to_string() }, Consumer { id: 2, title: "BB".to_string() }];
        let dummy_resources = vec![Resource { id: 1 }, Resource { id: 2 }];

        {
                let dummy_paginated_users_macro =
                        PaginatedUserResponse { page: dummy_pageinfo.clone(), data: dummy_users.clone() };
                let dummy_paginated_consumers_macro =
                        PaginatedConsumerResponse { page: dummy_pageinfo.clone(), data: dummy_consumers.clone() };
                let dummy_paginated_resources_macro =
                        PaginatedResourceResponse { page: dummy_pageinfo.clone(), data: dummy_resources.clone() };
                println!(
                        "dummy_paginated_users_macro JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_users_macro)
                                .unwrap()
                                .green()
                );
                println!(
                        "dummy_paginated_consumers_macro JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_consumers_macro)
                                .unwrap()
                                .blue()
                );
                println!(
                        "dummy_paginated_resources_macro JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_resources_macro)
                                .unwrap()
                                .cyan()
                );
        }

        {
                let dummy_paginated_users_tvar = PaginatedResponseAlt::<Users> {
                        page: dummy_pageinfo.clone(),
                        data: Users(dummy_users.clone()),
                };
                let dummy_paginated_consumers_tvar = PaginatedResponseAlt::<Consumers> {
                        page: dummy_pageinfo.clone(),
                        data: Consumers(dummy_consumers.clone()),
                };
                let dummy_paginated_resources_tvar = PaginatedResponseAlt::<Resources> {
                        page: dummy_pageinfo.clone(),
                        data: Resources(dummy_resources.clone()),
                };
                println!(
                        "tvar alt JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_users_tvar).unwrap().red()
                );
                println!(
                        "tvar alt JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_consumers_tvar)
                                .unwrap()
                                .yellow()
                );
                println!(
                        "tvar alt JSON:\n{}",
                        serde_json::to_string_pretty(&dummy_paginated_resources_tvar)
                                .unwrap()
                                .purple()
                );
        }
}
