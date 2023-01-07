use crossbeam::channel::{self, Receiver};
use log::warn;
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::record::Map;

#[derive(Debug, Serialize)]
pub struct Record {
    pub tag: &'static str,
    pub timestamp: u64,
    pub record: Map,
}

pub enum Message {
    Record(Record),
    Terminate,
}

pub struct Worker {
    conn: TcpStream,
    receiver: Receiver<Message>,
}

impl Worker {
    pub fn new(conn: TcpStream, receiver: Receiver<Message>) -> Self {
        Self { conn, receiver }
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Record(record)) => {
                    let mut buf = Vec::new();
                    match record.serialize(&mut Serializer::new(&mut buf)) {
                        Ok(_) => (),
                        Err(e) => warn!("failed to serialize a message: {}", e),
                    }
                    match self.conn.write_all(&buf).await {
                        Ok(_) => (),
                        Err(e) => warn!("failed to send a message to the fluent server: {}", e),
                    }
                }
                Err(channel::TryRecvError::Empty) => continue,
                Ok(Message::Terminate) | Err(channel::TryRecvError::Disconnected) => break,
            }
        }
    }
}
