//! A [fluentd](https://www.fluentd.org/) client using tokio.
//!
//! ## Example
//!
//! ```rust
//! use tokio_fluent::client::Client;
//! use tokio_fluent::entry::{Map, Value};
//!
//! let client = Client.new().await.unwrap();
//!
//! // With Map::new()
//! let mut map = Map::new();
//! map.insert("age".to_string(), 10.into());
//! client.send("fluent.test", map).unwrap();
//!
//! // With map! macro
//! let mut map_from_macro = map!(
//!   "age".to_string() => 22.into(),
//!   "scores".to_string() => [80, 90].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
//! );
//! ```

pub mod client;
pub mod entry;
mod worker;
