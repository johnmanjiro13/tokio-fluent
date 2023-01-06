use std::collections::HashMap;
use std::thread;

use tokio_fluent::client::{Client, Config};
use tokio_fluent::entry::{Map, Value};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let client = Client::new(&Config {
        addr: "127.0.0.1:24224".parse().unwrap(),
    })
    .await?;

    let mut m = HashMap::new();
    m.insert("Key".to_string(), "Value".into());
    m.insert("foo".to_string(), "bar".into());
    let mut map = Map::new_with(m);

    let mut map2 = Map::new();
    let v = vec![20, 20];
    map2.insert(
        "key".to_string(),
        v.into_iter()
            .map(|e| e.into())
            .collect::<Vec<Value>>()
            .into(),
    );

    map.insert("map".to_string(), map2.into());
    client.send("fluent.test", map).unwrap();

    let second = thread::spawn(move || {
        let mut map2 = Map::new();
        map2.insert("Key2".to_string(), Value::from("Value2".to_string()));
        client.send("client.test", map2).unwrap();
    });

    second.join().unwrap();
    Ok(())
}
