use crate::{
    account::AccountMap,
    data::UserData,
    matchmaking::{matchmake, process_queue, reconnect},
    queue::*,
    send::{Receivers, Sender},
    state::{AppState, SenderAppState},
    time::*,
    ws::ws_handler,
};
use axum::{extract::FromRef, routing::any, Router};
use bevy::prelude::{IntoScheduleConfigs, PluginGroup, System, Update};
use leptos::{logging, prelude::*, IntoView};
use leptos_axum::{file_and_error_handler, LeptosRoutes};
use session::Action;
use sqlx::*;

pub trait Register {
    fn register<U: UserData>(app: &mut bevy::prelude::App);
}

pub struct App {
    axum_router: Router,
    bevy_app: bevy::prelude::App,
}

impl App {
    pub fn new<Q, U, S, A, IV>(
        state: A,
        shell: fn(LeptosOptions) -> IV,
        routes: Vec<leptos_axum::AxumRouteListing>,
        pool: Pool<U::DB>,
    ) -> Self
    where
        A: AppState,
        Q: Queue + Register,
        U: UserData,
        S: Sender<UserData = U> + FromRef<SenderAppState<S, A>>,
        IV: IntoView + 'static,
        LeptosOptions: FromRef<A> + FromRef<SenderAppState<S, A>>,
    {
        let (sender, receivers) = S::new(pool);
        let state = SenderAppState::from_sender_and_options(sender, state);
        let axum_router = Router::new()
            .leptos_routes(&state, routes, {
                let leptos_options = LeptosOptions::from_ref(&state);
                move || shell(leptos_options.clone())
            })
            .route("/ws", any(ws_handler::<S>))
            .fallback(file_and_error_handler::<SenderAppState<S, A>, IV>(shell))
            .with_state(state);
        let mut bevy_app = bevy::prelude::App::new();
        bevy_app
            .add_plugins(bevy::prelude::MinimalPlugins.set(
                bevy::app::ScheduleRunnerPlugin::run_loop(core::time::Duration::from_secs_f64(
                    1.0 / 60.0,
                )),
            ))
            .insert_resource(AccountMap::default());
        Q::register::<U>(&mut bevy_app);
        receivers.insert(&mut bevy_app);
        Self {
            axum_router,
            bevy_app,
        }
    }
    pub fn add_time<T: AsStopwatch>(&mut self) {
        self.bevy_app.add_systems(Update, tick::<T>);
    }
    pub fn insert_resource<R: bevy::prelude::Resource>(&mut self, resource: R) {
        self.bevy_app.insert_resource(resource);
    }
    pub fn add_systems<M>(
        mut self,
        schedule: impl bevy::ecs::schedule::ScheduleLabel,
        systems: impl IntoScheduleConfigs<Box<dyn System<Out = (), In = ()>>, M>,
    ) -> Self {
        self.bevy_app.add_systems(schedule, systems);
        self
    }
    pub fn add_queue<QC: QueueComponent, U: UserData>(mut self) -> Self
    where
        QC::Action: Action<Shared = QC::Shared, User = QC::User>,
    {
        self.bevy_app.add_systems(Update, process_queue::<QC, U>);
        self
    }
    pub fn add_matchmake<QC: QueueComponent, U: UserData>(mut self) -> Self
    where
        QC::Action: Action<Shared = QC::Shared, User = QC::User>,
    {
        self.bevy_app.add_systems(Update, matchmake::<QC, U>);
        self
    }
    pub fn add_reconnect<QC: QueueComponent>(mut self) -> Self {
        self.bevy_app.add_systems(Update, reconnect::<QC>);
        self
    }
    pub fn run<IP>(self, addr: IP)
    where
        IP: 'static + Send + Sync + tokio::net::ToSocketAddrs + std::fmt::Display,
    {
        let Self {
            axum_router,
            mut bevy_app,
            ..
        } = self;

        tokio::spawn(async move {
            let router: Router = axum_router;
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            logging::log!("listening on http://{}", addr);
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });

        bevy_app.run();
    }
}
