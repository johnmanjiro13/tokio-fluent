use bytes::{Buf, BufMut};
use log::{error, warn};
use rmp_serde::Serializer;
use serde::{ser::SerializeMap, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::broadcast::{error::RecvError, Receiver},
    time::Duration,
};

use crate::record::Map;

const RETRY_INCREMENT_RATE: f64 = 1.5;

#[derive(Debug, Clone)]
pub enum Error {
    WriteFailed(String),
    ReadFailed(String),
    AckUnmatched(String, String),
    MaxRetriesExceeded,
    ConnectionClosed,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Error::WriteFailed(ref e) => e,
            Error::ReadFailed(ref e) => e,
            Error::AckUnmatched(_, _) => "request chunk and response ack did not match",
            Error::MaxRetriesExceeded => "max retries exceeded",
            Error::ConnectionClosed => "connection closed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub tag: String,
    pub timestamp: i64,
    pub record: Map,
    pub options: Options,
}

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub enum Message {
    Record(Record),
    Terminate,
}

#[derive(Debug)]
struct SerializedRecord {
    record: bytes::Bytes,
    chunk: String,
}

#[derive(Debug, Deserialize)]
struct AckResponse {
    ack: String,
}

pub struct RetryConfig {
    pub initial_wait: u64,
    pub max: u32,
    pub max_wait: u64,
}

pub struct Worker<StreamType> {
    stream: StreamType,
    receiver: Receiver<Message>,
    retry_config: RetryConfig,
}

impl<StreamType> Worker<StreamType>
where
    StreamType: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub fn new(stream: StreamType, receiver: Receiver<Message>, retry_config: RetryConfig) -> Self {
        Self {
            stream,
            receiver,
            retry_config,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.recv().await {
                Ok(Message::Record(record)) => {
                    let record = match self.encode(record) {
                        Ok(record) => record,
                        Err(e) => {
                            warn!("failed to serialize a message: {}", e);
                            continue;
                        }
                    };

                    match self.write_with_retry(&record).await {
                        Ok(_) => {}
                        Err(e) => match e {
                            Error::MaxRetriesExceeded => {
                                error!("Reached MaxRetriesExceeded");
                                break;
                            }
                            Error::ConnectionClosed => {
                                error!("Reached ConnectionClosed");
                                break;
                            }
                            _ => continue,
                        },
                    };
                }
                Err(RecvError::Closed) | Ok(Message::Terminate) => {
                    break;
                }
                Err(RecvError::Lagged(_)) => continue,
            }
        }
    }

    fn encode(&self, record: Record) -> Result<SerializedRecord, rmp_serde::encode::Error> {
        let mut writer = bytes::BytesMut::new().writer();
        record.serialize(&mut Serializer::new(&mut writer))?;
        Ok(SerializedRecord {
            record: writer.into_inner().freeze(),
            chunk: record.options.chunk,
        })
    }

    async fn write_with_retry(&mut self, record: &SerializedRecord) -> Result<(), Error> {
        let mut wait_time = Duration::from_millis(0);
        for i in 0..self.retry_config.max as i32 {
            tokio::time::sleep(wait_time).await;

            match self.write(record).await {
                Ok(_) => return Ok(()),
                Err(Error::ConnectionClosed) => return Err(Error::ConnectionClosed),
                Err(e) => {
                    warn!("Received error when writing: {:?}", e.to_string());
                }
            }

            let mut t =
                (self.retry_config.initial_wait as f64 * RETRY_INCREMENT_RATE.powi(i - 1)) as u64;
            if t > self.retry_config.max_wait {
                t = self.retry_config.max_wait;
            }
            wait_time = Duration::from_millis(t);
        }
        warn!("Write's max retries exceeded.");
        Err(Error::MaxRetriesExceeded)
    }

    async fn write(&mut self, record: &SerializedRecord) -> Result<(), Error> {
        match self.stream.write_all(record.record.chunk()).await {
            Ok(_) => {
                let received_ack = self.read_ack().await?;

                if received_ack.ack != record.chunk {
                    warn!(
                        "ack and chunk did not match. ack: {}, chunk: {}",
                        received_ack.ack, record.chunk
                    );
                    Err(Error::AckUnmatched(received_ack.ack, record.chunk.clone()))
                } else {
                    Ok(())
                }
            }
            // lost connection could look like multpile kinds of errors,
            // so we're attempting to catch all of them here
            Err(e)
                if e.kind() == std::io::ErrorKind::ConnectionReset
                    || e.kind() == std::io::ErrorKind::ConnectionAborted
                    || e.kind() == tokio::io::ErrorKind::BrokenPipe =>
            {
                Err(Error::ConnectionClosed)
            }
            Err(e) => Err(Error::WriteFailed(e.to_string())),
        }
    }

    async fn read_ack(&mut self) -> Result<AckResponse, Error> {
        let mut buf = bytes::BytesMut::with_capacity(64);
        loop {
            if let Ok(ack) = rmp_serde::from_slice::<AckResponse>(&buf) {
                return Ok(ack);
            }
            if self
                .stream
                .read_buf(&mut buf)
                .await
                .map_err(|e| Error::ReadFailed(e.to_string()))?
                == 0
            {
                return Err(Error::ConnectionClosed);
            }
        }
    }
}
