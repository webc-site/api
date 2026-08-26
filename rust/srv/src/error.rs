use std::io;
use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Wasmtime(#[from] wasmtime::Error),
    #[error("{0}")]
    Custom(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;
