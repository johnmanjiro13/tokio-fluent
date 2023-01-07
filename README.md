# tokio-fluent

A [fluentd](https://www.fluentd.org/) client using tokio.

## Example

```rust
use tokio_fluent::client::{Client, Config};
use tokio_fluent::entry::{Map, Value};
use tokio_fluent::map;

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

// With map! macro
let mut map_from_macro = map!(
     "age".to_string() => 22.into(),
     "scores".to_string() => [80, 90].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
);
```
