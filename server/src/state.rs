use crate::{data::UserData, send::Sender};
use axum::extract::FromRef;
use leptos::prelude::*;
use session::queue::Queue;

pub trait AppState: 'static + Clone + Send + Sync
where
    LeptosOptions: FromRef<Self>,
{
}

impl<T: 'static + Clone + Send + Sync> AppState for T where LeptosOptions: FromRef<T> {}

#[derive(Clone, Debug)]
pub struct SenderAppState<Q, U, App>
where
    Q: Queue,
    U: UserData,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    sender: Sender<Q, U>,
    pub user_defined: App,
}

impl<Q, U, App> FromRef<SenderAppState<Q, U, App>> for Sender<Q, U>
where
    Q: Queue,
    U: UserData,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    fn from_ref(input: &SenderAppState<Q, U, App>) -> Self {
        input.sender.clone()
    }
}

impl<Q, U, App> SenderAppState<Q, U, App>
where
    Q: Queue,
    U: UserData,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    pub fn from_sender_and_options(sender: Sender<Q, U>, user_defined: App) -> Self {
        Self {
            sender,
            user_defined,
        }
    }
}

impl<Q, U, App> FromRef<SenderAppState<Q, U, App>> for LeptosOptions
where
    Q: Queue,
    U: UserData,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    fn from_ref(input: &SenderAppState<Q, U, App>) -> Self {
        LeptosOptions::from_ref(&input.user_defined)
    }
}
