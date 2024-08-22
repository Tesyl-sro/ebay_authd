use crate::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Request {
    Status,
    Token,
    ForceRefresh,
    Stop,
}

impl From<Request> for Message {
    fn from(value: Request) -> Self {
        Self::Request(value)
    }
}
