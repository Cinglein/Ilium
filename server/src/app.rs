use crate::{
    account::AccountMap,
    data::UserData,
    matchmaking::{matchmake, process_queue, reconnect},
    send::Sender,
    state::{AppState, SenderAppState},
    time::tick,
    ws::ws_handler,
};
use axum::{extract::FromRef, routing::get, Router};
use bevy::prelude::{PluginGroup, Update};
use leptos::{logging, prelude::*, IntoView};
use leptos_axum::{file_and_error_handler, LeptosRoutes};
use session::queue::{Queue, QueueComponent};
use sqlx::*;

pub struct App {
    axum_router: Router,
    bevy_app: bevy::prelude::App,
}

impl App {
    pub fn new<Q, U, A, IV>(
        state: A,
        shell: fn(LeptosOptions) -> IV,
        routes: Vec<leptos_axum::AxumRouteListing>,
        pool: Pool<U::DB>,
    ) -> Self
    where
        A: AppState,
        Q: Queue,
        U: UserData,
        IV: IntoView + 'static,
        LeptosOptions: FromRef<A> + FromRef<SenderAppState<Q, U, A>>,
    {
        let (sender, receivers) = Sender::<Q, U>::new(pool);
        let state = SenderAppState::from_sender_and_options(sender, state);
        let axum_router = Router::new()
            .leptos_routes(&state, routes, {
                let leptos_options = LeptosOptions::from_ref(&state);
                move || shell(leptos_options.clone())
            })
            .route("/ws", get(ws_handler::<Q, U>))
            .fallback(file_and_error_handler::<SenderAppState<Q, U, A>, IV>(shell))
            .with_state(state);
        let mut bevy_app = bevy::prelude::App::new();
        bevy_app
            .add_plugins(bevy::prelude::MinimalPlugins.set(
                bevy::app::ScheduleRunnerPlugin::run_loop(bevy::utils::Duration::from_secs_f64(
                    1.0 / 60.0,
                )),
            ))
            .add_systems(Update, tick)
            .insert_resource(AccountMap::default());
        Q::register(&mut bevy_app);
        receivers.insert(&mut bevy_app);
        Self {
            axum_router,
            bevy_app,
        }
    }
    pub fn add_systems<S>(
        mut self,
        schedule: impl bevy::ecs::schedule::ScheduleLabel,
        systems: impl bevy::prelude::IntoSystemConfigs<S>,
    ) -> Self {
        self.bevy_app.add_systems(schedule, systems);
        self
    }
    pub fn add_queue<QC: QueueComponent, U: UserData>(mut self) -> Self {
        self.bevy_app.add_systems(Update, process_queue::<QC, U>);
        self
    }
    pub fn add_matchmake<QC: QueueComponent, U: UserData>(mut self) -> Self {
        self.bevy_app.add_systems(Update, matchmake::<QC, U>);
        self
    }
    pub fn add_reconnect<QC: QueueComponent>(mut self) -> Self {
        self.bevy_app.add_systems(Update, reconnect::<QC>);
        self
    }
    pub async fn run<IP>(self, addr: IP)
    where
        IP: 'static + Send + Sync + tokio::net::ToSocketAddrs + std::fmt::Display,
    {
        let Self {
            axum_router,
            mut bevy_app,
            ..
        } = self;
        let router: Router = axum_router;

        tokio::spawn(async move {
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
