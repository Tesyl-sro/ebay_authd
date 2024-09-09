use confy::ConfyError;
use std::{io, string::FromUtf8Error};
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

    #[error("Stop requested (not an error)")]
    StopRequested,

    #[error("Unexpected response")]
    UnexpectedResponse,

    #[error("Error while performing syscall: {0}")]
    Syscall(#[from] nix::errno::Errno),

    #[error("Client error: {0}")]
    Client(#[from] ebay_authd_client::error::Error),

    #[error("Error executing `screen`")]
    Screen,

    #[error("Failed to convert to UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;
