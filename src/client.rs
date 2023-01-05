use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::anyhow;
use crossbeam::channel::{self, Sender};
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::record::Record;

enum Message {
    Record(Record),
    Terminate,
}

pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    pub async fn new() -> anyhow::Result<Client> {
        let mut socket = TcpStream::connect("127.0.0.1:24224").await?;
        let (sender, receiver) = channel::unbounded();

        let _ = tokio::spawn(async move {
            loop {
                match receiver.try_recv() {
                    Ok(Message::Record(record)) => {
                        let mut buf = Vec::new();
                        // TODO: Print a log when an error is occurred.
                        record.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        socket.write_all(&buf).await?;
                    }
                    Err(channel::TryRecvError::Empty) => continue,
                    Ok(Message::Terminate) | Err(channel::TryRecvError::Disconnected) => break,
                }
            }

            Ok::<_, anyhow::Error>(())
        });

        Ok(Client { sender })
    }

    pub fn send(&self, tag: &'static str, entry: HashMap<String, String>) -> anyhow::Result<()> {
        let record = Record {
            tag,
            entry,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        };
        self.sender
            .send(Message::Record(record))
            .map_err(|e| anyhow!(e))
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        self.sender.send(Message::Terminate).map_err(|e| anyhow!(e))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Terminate);
    }
}
