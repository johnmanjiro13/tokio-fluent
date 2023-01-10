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
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use crossbeam::channel::{self, Sender};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::record::Map;
use crate::worker::{Message, Record, Worker};

#[derive(Debug, Clone)]
/// Config for a client.
pub struct Config {
    /// The address of the fluentd server.
    pub addr: SocketAddr,
    /// The timeout value to connect to the fluentd server.
    pub timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:24224".parse().unwrap(),
            timeout: Duration::new(3, 0),
        }
    }
}

#[async_trait]
pub trait FluentClient: Send + Sync {
    fn send(&self, tag: &'static str, record: Map) -> Result<(), Box<dyn std::error::Error>>;
    async fn stop(self) -> Result<(), channel::SendError<Message>>;
}

#[derive(Debug, Clone)]
/// A fluentd client.
pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    /// Connect to the fluentd server and create a worker with tokio::spawn.
    pub async fn new(config: &Config) -> tokio::io::Result<Client> {
        let socket = timeout(config.timeout, TcpStream::connect(config.addr)).await??;
        let (sender, receiver) = channel::unbounded();

        let _ = tokio::spawn(async move {
            let mut worker = Worker::new(socket, receiver);
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
    fn send(&self, tag: &'static str, record: Map) -> Result<(), Box<dyn std::error::Error>> {
        let record = Record {
            tag,
            record,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        };
        self.sender.send(Message::Record(record))?;
        Ok(())
    }

    /// Stop the worker.
    async fn stop(self) -> Result<(), channel::SendError<Message>> {
        self.sender.send(Message::Terminate)
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
    fn send(&self, _tag: &'static str, _record: Map) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn stop(self) -> Result<(), channel::SendError<Message>> {
        Ok(())
    }
}
