use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, time::SystemTime};

use anyhow::anyhow;
use crossbeam::channel::{self, Sender};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    tag: &'static str,
    timestamp: u64,
    entry: HashMap<String, String>,
}

struct Client {
    sender: Sender<Record>,
    worker: JoinHandle<io::Result<()>>,
}

impl Client {
    async fn new() -> io::Result<Client> {
        let mut socket = TcpStream::connect("127.0.0.1:24224").await?;
        let (sender, receiver) = channel::unbounded::<Record>();

        let worker = tokio::spawn(async move {
            loop {
                match receiver.try_recv() {
                    Ok(record) => {
                        let mut buf = Vec::new();
                        record.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        socket.write_all(&buf).await?;
                    }
                    Err(channel::TryRecvError::Empty) => continue,
                    Err(channel::TryRecvError::Disconnected) => break,
                }
            }

            Ok::<_, io::Error>(())
        });

        Ok(Client { sender, worker })
    }

    fn send(&self, tag: &'static str, entry: HashMap<String, String>) -> anyhow::Result<()> {
        let record = Record {
            tag,
            entry,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        };
        self.sender.send(record).map_err(|e| anyhow!(e))
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
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
