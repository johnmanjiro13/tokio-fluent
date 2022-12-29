use std::{collections::HashMap, time::SystemTime};

use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    timestamp: u64,
    entry: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    tag: &'static str,
    entries: Vec<Record>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:24224").await?;

    let send_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        let record = Record {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entry: map,
        };
        let message = Message {
            tag: "client.test",
            entries: vec![record],
        };
        message.serialize(&mut Serializer::new(&mut buf)).unwrap();
        socket.write_all(&buf).await?;

        Ok::<_, io::Error>(())
    });

    send_task.await?;
    Ok(())
}
