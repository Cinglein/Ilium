use crate::Action;

/// Everything that implements AsQueue on the client should implement Queue on the server
pub trait AsQueue: Send + Sync + 'static {
    type Action: Action;
}
