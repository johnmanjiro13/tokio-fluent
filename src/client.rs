//! Fluentd client
//!
//! ## Example
//!
//! ```no_run
//! use tokio_fluent::client::{Client, Config, FluentClient};
//! use tokio_fluent::record::{Map, Value};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new(&Config {
//!         addr: "127.0.0.1:24224".parse().unwrap(),
//!         ..Default::default()
//!     })
//!     .await
//!     .unwrap();
//!
//!     let mut map = Map::new();
//!     map.insert("age".to_string(), 10.into());
//!     client.send("fluent.test", map).unwrap();
//! }
//! ```

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use crossbeam::channel::{self, Sender};
use tokio::{net::TcpStream, sync::Mutex, time::timeout};
use uuid::Uuid;

use crate::error::ClientError;
use crate::record::Map;
use crate::worker::{Message, Options, Record, RetryConfig, Worker};

#[derive(Debug, Clone)]
/// Config for a client.
pub struct Config {
    /// The address of the fluentd server.
    /// The default is `127.0.0.1:24224`.
    pub addr: SocketAddr,
    /// The timeout value to connect to the fluentd server.
    /// The default is 3 seconds.
    pub timeout: Duration,
    /// The duration of the initial wait for the first retry, in milliseconds.
    /// The default is 500.
    pub retry_wait: u64,
    /// The maximum number of retries. If the number of retries become larger
    /// than this value, the write/send operation will fail. The default is 13.
    pub max_retry: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:24224".parse().unwrap(),
            timeout: Duration::new(3, 0),
            retry_wait: 500,
            max_retry: 13,
        }
    }
}

#[async_trait]
pub trait FluentClient: Send + Sync {
    fn send(&self, tag: &'static str, record: Map) -> Result<(), ClientError>;
    async fn stop(self) -> Result<(), ClientError>;
}

#[derive(Debug, Clone)]
/// A fluentd client.
pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    /// Connect to the fluentd server and create a worker with tokio::spawn.
    pub async fn new(config: &Config) -> tokio::io::Result<Client> {
        let stream = timeout(config.timeout, TcpStream::connect(config.addr)).await??;
        let (sender, receiver) = channel::unbounded();

        let config = config.clone();
        let _ = tokio::spawn(async move {
            let worker = Worker::new(
                Arc::new(Mutex::new(stream)),
                receiver,
                RetryConfig {
                    initial_wait: config.retry_wait,
                    max: config.max_retry,
                },
            );
            worker.run().await
        });

        Ok(Self { sender })
    }
}

#[async_trait]
impl FluentClient for Client {
    /// Send a fluent record to the fluentd server.
    ///
    /// ## Params:
    /// `tag` - Event category of a record to send.
    ///
    /// `record` - Map object to send as a fluent record.
    fn send(&self, tag: &'static str, record: Map) -> Result<(), ClientError> {
        let record = Record {
            tag,
            record,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| ClientError::DeriveError(e.to_string()))?
                .as_secs(),
            options: Options {
                chunk: general_purpose::STANDARD.encode(Uuid::new_v4().to_string()),
            },
        };
        self.sender
            .send(Message::Record(record))
            .map_err(|e| ClientError::SendError(e.to_string()))?;
        Ok(())
    }

    /// Stop the worker.
    async fn stop(self) -> Result<(), ClientError> {
        self.sender
            .send(Message::Terminate)
            .map_err(|e| ClientError::SendError(e.to_string()))
    }
}

/// The worker is terminated when client is dropped.
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Terminate);
    }
}

#[derive(Debug, Clone)]
/// NopClient does nothing.
pub struct NopClient;

#[async_trait]
impl FluentClient for NopClient {
    fn send(&self, _tag: &'static str, _record: Map) -> Result<(), ClientError> {
        Ok(())
    }

    async fn stop(self) -> Result<(), ClientError> {
        Ok(())
    }
}
