use std::{error::Error as StdError, fmt, io::Error as IoError, result::Result as StdResult};

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Redis(String),
    Protocol(String),
    ConnectionClosed,
    InvalidResponse,
    ClusterSlotUncovered(u16),
    Moved { slot: u16, addr: String },
    Ask { slot: u16, addr: String },
    Sentinel(String),
    Config(String),
    Timeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Redis(msg) => write!(f, "redis error: {msg}"),
            Self::Protocol(msg) => write!(f, "protocol error: {msg}"),
            Self::ConnectionClosed => write!(f, "connection closed"),
            Self::InvalidResponse => write!(f, "invalid response"),
            Self::ClusterSlotUncovered(slot) => write!(f, "cluster slot {slot} not covered"),
            Self::Moved { slot, addr } => write!(f, "MOVED {slot} to {addr}"),
            Self::Ask { slot, addr } => write!(f, "ASK {slot} to {addr}"),
            Self::Sentinel(msg) => write!(f, "sentinel error: {msg}"),
            Self::Config(msg) => write!(f, "config error: {msg}"),
            Self::Timeout => write!(f, "operation timeout"),
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => Self::Io(IoError::new(e.kind(), e.to_string())),
            Self::Redis(msg) => Self::Redis(msg.clone()),
            Self::Protocol(msg) => Self::Protocol(msg.clone()),
            Self::ConnectionClosed => Self::ConnectionClosed,
            Self::InvalidResponse => Self::InvalidResponse,
            Self::ClusterSlotUncovered(slot) => Self::ClusterSlotUncovered(*slot),
            Self::Moved { slot, addr } => Self::Moved {
                slot: *slot,
                addr: addr.clone(),
            },
            Self::Ask { slot, addr } => Self::Ask {
                slot: *slot,
                addr: addr.clone(),
            },
            Self::Sentinel(msg) => Self::Sentinel(msg.clone()),
            Self::Config(msg) => Self::Config(msg.clone()),
            Self::Timeout => Self::Timeout,
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = StdResult<T, Error>;
