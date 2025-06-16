use crate::send::Sender;
use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;

pub trait AppState: 'static + Clone + Send + Sync
where
    LeptosOptions: FromRef<Self>,
{
}

impl<T: 'static + Clone + Send + Sync> AppState for T where LeptosOptions: FromRef<T> {}

#[derive(Clone, Debug)]
pub struct SenderAppState<S, App>
where
    S: Sender,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    pub sender: S,
    pub user_defined: App,
}

impl<S, App> SenderAppState<S, App>
where
    S: Sender,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    pub fn from_sender_and_options(sender: S, user_defined: App) -> Self {
        Self {
            sender,
            user_defined,
        }
    }
}

impl<S, App> FromRef<SenderAppState<S, App>> for LeptosOptions
where
    S: Sender,
    App: AppState,
    LeptosOptions: FromRef<App>,
{
    fn from_ref(input: &SenderAppState<S, App>) -> Self {
        LeptosOptions::from_ref(&input.user_defined)
    }
}
