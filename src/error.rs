#[derive(Debug)]
pub enum Error {
    DeriveError(String),
    SendError(String),
    UnmatchedError(String, String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Error::DeriveError(ref e) => e,
            Error::SendError(ref e) => e,
            Error::UnmatchedError(_, _) => "request chunk and response ack did not match",
        };
        write!(f, "{}", s)
    }
}
