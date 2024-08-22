#![allow(
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc
)]

use serde::{Deserialize, Serialize};
pub use serde_json::Error as SerializeError;

pub mod request;
pub mod response;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Message {
    Request(request::Request),
    Response(response::Response),
}

impl Message {
    pub fn serialize(self) -> serde_json::Result<Box<str>> {
        Ok(serde_json::to_string(&self)?.into())
    }

    pub fn deserialize(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    #[must_use]
    pub fn into_response(self) -> Option<response::Response> {
        match self {
            Self::Request(..) => None,
            Self::Response(resp) => Some(resp),
        }
    }

    #[must_use]
    pub fn into_request(self) -> Option<request::Request> {
        match self {
            Self::Request(req) => Some(req),
            Self::Response(..) => None,
        }
    }
}
