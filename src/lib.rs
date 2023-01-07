//! A [fluentd](https://www.fluentd.org/) client using tokio.
//!
//! ## Example
//!
//! ```no_run
//! use std::collections::HashMap;
//!
//! use tokio_fluent::record_map;
//! use tokio_fluent::client::{Client, Config, FluentClient};
//! use tokio_fluent::record::{Map, Value};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new(&Config { addr: "127.0.0.1:24224".parse().unwrap() }).await.unwrap();
//!
//!     // With Map::new()
//!     let mut map = Map::new();
//!     map.insert("age".to_string(), 10.into());
//!     client.send("fluent.test", map).unwrap();
//!
//!     // With record_map! macro
//!     let mut map_from_macro = record_map!(
//!       "age".to_string() => 22.into(),
//!       "scores".to_string() => [80, 90].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
//!     );
//! }
//! ```

pub mod client;
pub mod record;
mod worker;
