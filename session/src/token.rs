use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Archive, Serialize, Deserialize)]
pub enum ClientToken {
    Guest,
}
