use std::sync::Arc;

use crossbeam::channel::{self, Receiver};
use log::warn;
use rmp_serde::{Deserializer, Serializer};
use serde::{ser::SerializeMap, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};
use tokio_retry::{strategy::ExponentialBackoff, Retry};

use crate::{error::Error, record::Map};

#[derive(Debug, Serialize)]
pub struct Record {
    pub tag: &'static str,
    pub timestamp: u64,
    pub record: Map,
    pub options: Options,
}

#[derive(Debug)]
pub struct Options {
    pub chunk: String,
}

impl Serialize for Options {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("chunk", &self.chunk)?;
        map.end()
    }
}

pub enum Message {
    Record(Record),
    Terminate,
}

#[derive(Debug, Deserialize)]
struct Response {
    ack: String,
}

pub struct RetryConfig {
    pub initial_wait: u64,
    pub max: usize,
}

pub struct Worker {
    stream: Arc<Mutex<TcpStream>>,
    receiver: Receiver<Message>,
    retry_config: RetryConfig,
}

impl Worker {
    pub fn new(
        stream: Arc<Mutex<TcpStream>>,
        receiver: Receiver<Message>,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            stream,
            receiver,
            retry_config,
        }
    }

    pub async fn run(&self) {
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Record(record)) => {
                    let mut buf = Vec::new();
                    match record.serialize(&mut Serializer::new(&mut buf)) {
                        Ok(_) => (),
                        Err(e) => {
                            warn!("failed to serialize a message: {}", e);
                            continue;
                        }
                    }

                    let retry_strategy =
                        ExponentialBackoff::from_millis(self.retry_config.initial_wait)
                            .take(self.retry_config.max);
                    let retry_task = Retry::spawn(retry_strategy, || async {
                        self.send(&buf, record.options.chunk.clone()).await
                    });

                    match retry_task.await {
                        Ok(_) => (),
                        Err(e) => {
                            warn!("failed to send a message to the fluent server: {}", e);
                            continue;
                        }
                    }
                }
                Err(channel::TryRecvError::Empty) => continue,
                Ok(Message::Terminate) | Err(channel::TryRecvError::Disconnected) => break,
            }
        }
    }

    async fn send(&self, src: &[u8], chunk: String) -> Result<(), Error> {
        let stream = self.stream.clone();
        let mut stream = stream.lock().await;
        stream
            .write_all(src)
            .await
            .map_err(|e| Error::DeriveError(e.to_string()))?;

        let mut buf = vec![0; 128];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| Error::DeriveError(e.to_string()))?;
        let response: Response =
            Deserialize::deserialize(&mut Deserializer::new(&buf[0..n])).unwrap();
        if response.ack == chunk {
            Ok(())
        } else {
            warn!(
                "ack and chunk did not match. ack: {}, chunk: {}",
                response.ack, chunk
            );
            Err(Error::UnmatchedError(response.ack, chunk))
        }
    }
}
