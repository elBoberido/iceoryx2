use serde::{Deserialize, Serialize};

pub const SERVICE_CONNECTION_REQUEST_PORT: u16 = 0xA2D2;

pub const SERVICE_ID: &str = "ping-pong";

pub struct ProtocolVersion {
    version: u32,
}

impl ProtocolVersion {
    pub fn entity() -> Self {
        ProtocolVersion {
            version: env!("CARGO_PKG_VERSION_MAJOR")
                .parse::<u32>()
                .unwrap_or(std::u32::MAX),
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Request {
    Ping(u64),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Response {
    Pong(u64),
}

pub type Error = String;
