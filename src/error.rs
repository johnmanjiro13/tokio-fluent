use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    DeriveError(String),
    SendError(String),
}

impl std::error::Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            ClientError::DeriveError(ref e) => e,
            ClientError::SendError(ref e) => e,
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum WorkerError {
    DeriveError(String),
    AckUnmatchedError(String, String),
}

impl std::error::Error for WorkerError {}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            WorkerError::DeriveError(ref e) => e,
            WorkerError::AckUnmatchedError(_, _) => "request chunk and response ack did not match",
        };
        write!(f, "{}", s)
    }
}
