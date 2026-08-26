#[cfg(not(target_arch = "wasm32"))]
use std::io;
use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Io(#[from] io::Error),
    #[cfg(target_arch = "wasm32")]
    #[error(transparent)]
    Http(#[from] http::Error),
    #[cfg(target_arch = "wasm32")]
    #[error("{0}")]
    Custom(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;
