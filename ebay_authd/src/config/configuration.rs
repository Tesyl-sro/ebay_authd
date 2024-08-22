use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Configuration {
    pub appid: Box<str>,
    pub devid: Box<str>,
    pub certid: Box<str>,
    pub redirecturi: Box<str>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            appid: "".into(),
            devid: "".into(),
            certid: "".into(),
            redirecturi: "".into(),
        }
    }
}
