use crossbeam::channel::{self, Receiver};
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::record::Record;

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
        Worker { conn, receiver }
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Record(record)) => {
                    let mut buf = Vec::new();
                    // TODO: Print a log when an error is occurred.
                    record.serialize(&mut Serializer::new(&mut buf)).unwrap();
                    self.conn.write_all(&buf).await.unwrap();
                }
                Err(channel::TryRecvError::Empty) => continue,
                Ok(Message::Terminate) | Err(channel::TryRecvError::Disconnected) => break,
            }
        }
    }
}
