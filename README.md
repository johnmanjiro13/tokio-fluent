# tokio-fluent

A [fluentd](https://www.fluentd.org/) client using tokio.

## Example

```rust
use tokio_fluent::client::Client;
use tokio_fluent::entry::{Map, Value};

let client = Client.new().await.unwrap();
let mut map = Map::new();
map.insert("age".to_string(), 10.into());
client.send("fluent.test", map).unwrap();
```
