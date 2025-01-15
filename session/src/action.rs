use crate::{msg::Message, state::*};

pub trait Action: Sized + Copy + Message {
    type Shared: SharedState;
    type User: UserState;
    fn resolve(action: Option<Self>, shared: &mut Self::Shared, users: &mut [Self::User]);
}
