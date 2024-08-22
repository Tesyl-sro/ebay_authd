use confy::ConfyError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not find home directory path of current user")]
    NoHome,

    #[error("Config error: {0}")]
    Config(#[from] ConfyError),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] oauth2::url::ParseError),

    #[error("I/O: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to exchange auth code for token")]
    TokenRequest,

    #[error("Serialization: {0}")]
    Serialize(#[from] ebay_authd_core::SerializeError),

    #[error("Stop requested (not an error)")]
    StopRequested,

    #[error("Expected response, got request")]
    ExpectedResponse,

    #[error("Unexpected response")]
    UnexpectedResponse,

    #[error("Error while performing syscall: {0}")]
    Syscall(#[from] nix::errno::Errno),
}

pub type Result<T> = ::std::result::Result<T, Error>;
