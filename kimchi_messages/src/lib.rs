use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum KimchiMessage {
    Joined {
        user: String,
    },
    Public {
        user: String,
        message: String,
    },
    Private {
        user: String,
        to_user: String,
        message: String,
    },
    Video {
        user: String,
        data: Vec<u8>,
    }
}

impl KimchiMessage {
    pub fn from_str(msg: &str) -> Option<KimchiMessage> {
        serde_json::from_str(msg).ok()
    }

    pub fn to_str(msg: &KimchiMessage) -> Option<String> {
        serde_json::to_string(msg).ok()
    }
}
