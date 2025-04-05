use serde::{Deserialize, Serialize};

use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct SocketMessage {
    pub event: String,
    pub public_key: Option<String>,
    pub identifier: Option<String>,
    pub signature: Option<String>,
    pub data: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub identifier: String,
    pub data: String,
    pub public_key: String,
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct FinalResponse {
    pub data: Option<Response>,
    pub error: Option<String>
}

pub enum Event {
    GET,
    PUT,
    INVALID,
}

impl FromStr for Event {
    type Err = ();

    fn from_str(input: &str) -> Result<Event, Self::Err> {
        match input {
            "GET" => Ok(Event::GET),
            "PUT" => Ok(Event::PUT),
            _ => Ok(Event::INVALID),
        }
    }
}
