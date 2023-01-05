use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use fluentd_client_rs::client::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new().await?;
    let mut map = HashMap::new();
    map.insert("Key".to_string(), "Value".to_string());
    client.send("fluent.test", map).unwrap();

    sleep(Duration::new(3, 0));

    let mut map2 = HashMap::new();
    map2.insert("Key2".to_string(), "Value2".to_string());
    client.send("client.test", map2).unwrap();
    Ok(())
}
