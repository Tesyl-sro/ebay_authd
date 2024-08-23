use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization: {0}")]
    Serialize(#[from] ebay_authd_core::SerializeError),

    #[error("Expected response, got request")]
    ExpectedResponse,

    #[error("Broken connection pipe")]
    BrokenConnection,
}

pub type Result<T> = ::std::result::Result<T, Error>;
