use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Decode, Encode)]
pub enum ClientToken {
    Guest,
}
