# tokio-fluent

A [fluentd](https://www.fluentd.org/) client using tokio.

## Example

```rust
use tokio_fluent::client::{Client, Config};
use tokio_fluent::entry::{Map, Value};

let client = Client::new(&Config { addr: "127.0.0.1:24224".parse().unwrap() }).await.unwrap();
let mut map = Map::new();
map.insert("age".to_string(), 10.into());
map.insert(
    "scores".to_string(),
    vec![80, 90]
        .into_iter()
        .map(|e| e.into())
        .collect::<Vec<Value>>()
        .into(),
);
client.send("fluent.test", map).unwrap();
```
