use crate::{msg::Message, state::*};

pub trait Action: Sized + Copy + Message {
    type Shared: SharedState;
    type User: UserState;
    fn update<S: AsState<User = Self::User, Shared = Self::Shared>>(state: S);
    fn resolve<S: AsState<User = Self::User, Shared = Self::Shared>>(
        self,
        index: S::Index,
        state: S,
    );
}
