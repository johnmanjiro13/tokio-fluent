# tokio-fluent

[![Crates.io](https://img.shields.io/crates/v/tokio-fluent.svg)](https://crates.io/crates/tokio-fluent)
[![Documentation](https://docs.rs/tokio-fluent/badge.svg)](https://docs.rs/crate/tokio-fluent/)
[![CI](https://github.com/johnmanjiro13/tokio-fluent/workflows/test/badge.svg?branch%3Amain)](https://github.com/johnmanjiro13/tokio-fluent/actions?query=workflow%3Atest%20branch%3Amain)


A [fluentd](https://www.fluentd.org/) client using [tokio](https://tokio.rs/).

## Installation

Add this to your `Cargo.toml`

```toml
[dependencies]
tokio-fluent = "0.2.1"
```

## Example

```rust
use std::collections::HashMap;

use tokio_fluent::client::{Client, Config, FluentClient};
use tokio_fluent::record::{Map, Value};
use tokio_fluent::record_map;

#[tokio::main]
async fn main() {
    let client = Client::new(&Config { addr: "127.0.0.1:24224".parse().unwrap() }).await.unwrap();

    // With Map::new()
    let mut map = Map::new();
    map.insert("age".to_string(), 22.into());
    map.insert(
        "scores".to_string(),
        vec![80, 90]
            .into_iter()
            .map(|e| e.into())
            .collect::<Vec<_>>()
            .into(),
    );
    client.send("fluent.test", map).unwrap();

    // With record_map! macro
    let map_from_macro = record_map!(
        "age".to_string() => 22.into(),
        "scores".to_string() => [80, 90].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
    );
    client.send("fluent.test", map_from_macro).unwrap();
}
```

## Setting config values

```rust
let client = Client::new(&Config {
        addr: "127.0.0.1:24224".parse().unwrap(),
    })
    .await
    .unwrap();
```

### Timeout

Set the timeout value of `std::time::Duration` to connect to the destination. The default is 3 seconds.
