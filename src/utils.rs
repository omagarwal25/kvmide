use rdev::{Event, GrabError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Message(String),
    Command(Event),
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Ready,
}

#[derive(Debug, Error)]
pub enum RdevError {
    #[error("Issue with grabbing input")]
    Grab(GrabError),
}
