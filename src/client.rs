//! Fluentd client
//!
//! ## Example
//!
//! ```no_run
//! use tokio_fluent::client::{Client,Config};
//! use tokio_fluent::entry::{Map, Value};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::new(&Config { addr: "127.0.0.1:24224".parse().unwrap() }).await.unwrap();
//!     let mut map = Map::new();
//!     map.insert("age".to_string(), 10.into());
//!     client.send("fluent.test", map).unwrap();
//! }
//! ```

use std::net::SocketAddr;
use std::time::SystemTime;

use crossbeam::channel::{self, Sender};
use tokio::net::TcpStream;

use crate::entry::Map;
use crate::worker::{Message, Record, Worker};

#[derive(Debug, Clone)]
/// Config for a client.
pub struct Config {
    /// Address of the fluentd server.
    pub addr: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:24224".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
/// A fluentd client.
pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    /// Connect to the fluentd server and create a worker with tokio::spawn.
    pub async fn new(config: &Config) -> tokio::io::Result<Client> {
        let socket = TcpStream::connect(config.addr).await?;
        let (sender, receiver) = channel::unbounded();

        let _ = tokio::spawn(async move {
            let mut worker = Worker::new(socket, receiver);
            worker.run().await
        });

        Ok(Self { sender })
    }

    /// Send a fluent record to the fluentd server.
    ///
    /// ## Params:
    /// `tag` - Event category of a record to send.
    ///
    /// `entry` - Map object to send as a fluent record.
    pub fn send(&self, tag: &'static str, entry: Map) -> Result<(), Box<dyn std::error::Error>> {
        let record = Record {
            tag,
            entry,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        };
        self.sender.send(Message::Record(record))?;
        Ok(())
    }

    /// Stop the worker.
    pub async fn stop(self) -> Result<(), channel::SendError<Message>> {
        self.sender.send(Message::Terminate)
    }
}

/// The worker is terminated when client is dropped.
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Terminate);
    }
}
