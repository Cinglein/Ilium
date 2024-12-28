use crate::{msg::Message, state::State};

pub trait Action: Sized + Message {
    type Shared: State;
    type User: State;
    fn resolve(action: Option<Self>, shared: &mut Self::Shared, user: &mut [Self::User]);
}
