use crate::Message;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Response {
    Status { version: Box<str>, expiry: Duration },
    Token(Box<str>),
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}
