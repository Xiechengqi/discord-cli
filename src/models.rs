use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub index: u32,
    pub server: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub index: u32,
    pub channel: String,
    pub r#type: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub index: u32,
    pub name: String,
    pub status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub author: String,
    pub time: String,
    pub message: String,
}
