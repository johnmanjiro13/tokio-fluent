//! Fluentd client
//!
//! ## Example
//!
//! ```
//! use tokio_fluent::{Client, Config, FluentClient};
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
use std::time::Duration;

use base64::{engine::general_purpose, Engine};
use tokio::{
    net::TcpStream,
    sync::broadcast::{channel, Sender},
    time::timeout,
};
use uuid::Uuid;

use crate::record::Map;
use crate::worker::{Message, Options, Record, RetryConfig, Worker};

#[derive(Debug, Clone)]
pub struct SendError {
    source: String,
}

impl std::error::Error for SendError {}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

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
    /// than this value, the write/send operation will fail. The default is 10.
    pub max_retry: u32,
    /// The maximum duration of wait between retries, in milliseconds.
    /// If calculated retry wait is larger than this value, operation will fail.
    /// The default is 60,000 (60 seconds).
    pub max_retry_wait: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:24224".parse().unwrap(),
            timeout: Duration::new(3, 0),
            retry_wait: 500,
            max_retry: 10,
            max_retry_wait: 60000,
        }
    }
}

pub trait FluentClient: Send + Sync {
    fn send(&self, tag: &str, record: Map) -> Result<(), SendError>;
    fn stop(self) -> Result<(), SendError>;
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
        let (sender, receiver) = channel(1024);

        let config = config.clone();
        let _ = tokio::spawn(async move {
            let mut worker = Worker::new(
                stream,
                receiver,
                RetryConfig {
                    initial_wait: config.retry_wait,
                    max: config.max_retry,
                    max_wait: config.max_retry_wait,
                },
            );
            worker.run().await
        });

        Ok(Self { sender })
    }

    fn send_with_time(&self, tag: &str, record: Map, timestamp: i64) -> Result<(), SendError> {
        let record = Record {
            tag: tag.into(),
            record,
            timestamp,
            options: Options {
                chunk: general_purpose::STANDARD.encode(Uuid::new_v4()),
            },
        };
        self.sender
            .send(Message::Record(record))
            .map_err(|e| SendError {
                source: e.to_string(),
            })?;
        Ok(())
    }
}

impl FluentClient for Client {
    /// Send a fluent record to the fluentd server.
    ///
    /// ## Params:
    /// `tag` - Event category of a record to send.
    ///
    /// `record` - Map object to send as a fluent record.
    fn send(&self, tag: &str, record: Map) -> Result<(), SendError> {
        self.send_with_time(tag, record, chrono::Local::now().timestamp())
    }

    /// Stop the worker.
    fn stop(self) -> Result<(), SendError> {
        self.sender
            .send(Message::Terminate)
            .map_err(|e| SendError {
                source: e.to_string(),
            })?;
        Ok(())
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

impl FluentClient for NopClient {
    fn send(&self, _tag: &str, _record: Map) -> Result<(), SendError> {
        Ok(())
    }

    fn stop(self) -> Result<(), SendError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_with_time() {
        use std::collections::HashMap;

        use chrono::TimeZone;

        use crate::record::Value;
        use crate::record_map;

        let (sender, mut receiver) = channel(1024);
        let client = Client { sender };

        let timestamp = chrono::Utc.timestamp_opt(1234567, 0).unwrap().timestamp();
        let record = record_map!("age".to_string() => 20.into());
        assert!(
            client.send_with_time("test", record, timestamp).is_ok(),
            "failed to send with time"
        );

        let got = receiver.try_recv().expect("failed to receive");
        match got {
            Message::Record(r) => {
                assert_eq!(r.tag, "test");
                assert_eq!(r.record, record_map!("age".to_string() => 20.into()));
                assert_eq!(r.timestamp, 1234567);
            }
            Message::Terminate => unreachable!("got terminate message"),
        }
    }

    #[test]
    fn test_stop() {
        let (sender, mut receiver) = channel(1024);
        let client = Client { sender };
        assert!(client.stop().is_ok(), "faled to stop");

        let got = receiver.try_recv().expect("failed to receive");
        match got {
            Message::Record(_) => unreachable!("got record message"),
            Message::Terminate => {}
        };
    }

    #[test]
    fn test_client_drop_sends_terminate() {
        let (sender, mut receiver) = channel(1024);
        {
            Client { sender };
        }
        let got = receiver.try_recv().expect("failed to receive");
        match got {
            Message::Record(_) => unreachable!("got record message"),
            Message::Terminate => {}
        };
    }

    #[test]
    fn test_default_config() {
        let config: Config = Default::default();
        assert_eq!(config.addr, "127.0.0.1:24224".parse().unwrap());
        assert_eq!(config.timeout, Duration::new(3, 0));
        assert_eq!(config.retry_wait, 500);
        assert_eq!(config.max_retry, 10);
        assert_eq!(config.max_retry_wait, 60000);
    }
}
