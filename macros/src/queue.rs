use crate::util::*;
use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote};
use syn::*;

pub fn derive_queue_impl(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let queue: Ident = ast.ident;
    let action: Ident = name_value(&ast.attrs, "action")
        .unwrap_or_else(|| abort_call_site!("Could not find action attribute"));
    let mut variant_name: Vec<Ident> = Vec::new();
    let mut component: Vec<Ident> = Vec::new();
    let mut lobby_name: Vec<Ident> = Vec::new();
    let mut lobby_type: Vec<Type> = Vec::new();
    let mut queue_sender: Vec<Ident> = Vec::new();
    let mut queue_receiver: Vec<Ident> = Vec::new();
    let mut reconnect_sender: Vec<Ident> = Vec::new();
    let mut reconnect_receiver: Vec<Ident> = Vec::new();
    let mut action_sender: Vec<Ident> = Vec::new();
    let mut action_receiver: Vec<Ident> = Vec::new();

    match ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants.iter() {
                let component_name = format_ident!("{}Component", variant.ident);
                let name = format_ident!("{}Lobby", variant.ident);
                let size: LitInt = name_value(&variant.attrs, "size")
                    .unwrap_or_else(|| abort_call_site!("Could not find lobby size"));
                let ty: Type = parse_str(&format!("[::bevy::prelude::Entity; {}]", size))
                    .expect("Could not parse lobby");
                variant_name.push(variant.ident.clone());
                component.push(component_name);
                let lower = variant.ident.to_string().to_lowercase();
                queue_sender.push(format_ident!("{}_queue", lower));
                queue_receiver.push(format_ident!("{}_queue_recv", lower));
                reconnect_sender.push(format_ident!("{}_reconnect", lower));
                reconnect_receiver.push(format_ident!("{}_reconnect_recv", lower));
                action_sender.push(format_ident!("{}_action", lower));
                action_receiver.push(format_ident!("{}_action_recv", lower));
                lobby_name.push(name);
                lobby_type.push(ty);
            }
        }
        _ => abort_call_site!("Only enums are supported."),
    };

    let sender = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "server")] {
                let sender_name = format_ident!("{queue}Sender");
                let receivers_name = format_ident!("{queue}Receivers");
                quote! {
                    pub struct #receivers_name {
                        #(pub #queue_sender: ::ilium::server::send::Receiver<::ilium::server::send::QueueSignal<#component>>,)*
                        #(pub #reconnect_sender: ::ilium::server::send::Receiver<::ilium::server::send::ReconnectSignal<#component>>,)*
                        #(pub #action_sender: ::ilium::server::send::Receiver<::ilium::server::send::ActionSignal<#component>>,)*
                    }

                    impl ::ilium::server::send::Receivers for #receivers_name {
                        fn insert(self, app: &mut ::bevy::prelude::App) {
                            let Receivers {
                                #(#queue_sender,)*
                                #(#reconnect_sender,)*
                                #(#action_sender,)*
                            } = self;
                            #(app.insert_resource(#queue_sender);)*
                            #(app.insert_resource(#reconnect_sender);)*
                            #(app.insert_resource(#action_sender);)*
                        }
                    }

                    #[derive(Debug, Resource)]
                    pub struct #sender_name<U: ::ilium::server::data::UserData> {
                        pub pool: Pool<U::DB>,
                        #(pub #queue_sender: ::kanal::Sender<::ilium::server::send::QueueSignal<#component>>,)*
                        #(pub #reconnect_sender: ::kanal::Sender<::ilium::server::send::ReconnectSignal<#component>>,)*
                        #(pub #action_sender: ::kanal::Sender<::ilium::server::send::ActionSignal<#component>>,)*
                    }

                    impl<U: ::ilium::server::data::UserData> Clone for #sender_name<U> {
                        fn clone(&self) -> Self {
                            Self {
                                pool: self.pool.clone(),
                                #(#queue_sender: self.#queue_sender.clone(),)*
                                #(#reconnect_sender: self.#reconnect_sender.clone(),)*
                                #(#action_sender: self.#action_sender.clone(),)*
                            }
                        }
                    }

                    impl<App> ::axum::extract::FromRef<::ilium::server::state::SenderAppState<#sender_name, App>> for #sender_name
                    where
                        S: ::ilium::server::send::Sender,
                        App: ::ilium::server::state::AppState,
                        LeptosOptions: ::axum::extract::FromRef<App>,
                    {
                        fn from_ref(input: &::ilium::server::state::SenderAppState<S, App>) -> Self {
                            input.sender.clone()
                        }
                    }

                    impl<U: ilium::server::data::UserData> ::ilium::server::Sender for #sender_name<U> {
                        type Receivers = #receivers_name;
                        type Queue = #queue;
                        type UserData = U;
                        fn new(pool: Pool<U::DB>) -> (Self, Self::Receivers) {
                            #(let (#queue_sender, #queue_receiver) = ::kanal::unbounded();)*
                            #(let (#reconnect_sender, #action_receiver) = ::kanal::unbounded();)*
                            #(let (#action_sender, #action_receiver) = ::kanal::unbounded();)*
                            let sender = Self {
                                pool,
                                #(#queue_sender,)*
                                #(#reconnect_sender,)*
                                #(#action_sender,)*
                            };
                            let receivers = Self::Receivers {
                                #(#queue_sender: ::ilium::server::send::Receiver(#queue_receiver))*
                                #(#reconnect_sender: ::ilium::server::send::Receiver(#reconnect_receiver))*
                                #(#action_sender: ::ilium::server::send::Receiver(#action_receiver))*
                            };
                            (sender, receivers)
                        }
                         fn send(
                            &self,
                            msg: Msg<Self::Queue>,
                            ip: ::std::net::SocketAddr,
                            send_frame: ::ilium::server::send::SendFrame,
                            ping: ::ilium::server::time::Ping,
                        ) -> impl ::core::future::Future<Output = ::eyre::Result<()>> + Send + Sync {
                            let ::ilium::session::msg::Msg { token, queue, msg_type } = msg;
                            let account = ::ilium::server::auth::auth(token, ip);
                            match (msg_type, queue) {
                                #(
                                    (MsgType::Join, #queue::#variant_name) => {
                                        let user_data = U::query(&self.pool, &account).await?;
                                        self.#queue_sender.send(::ilium::server::send::QueueSignal::Join {
                                            account,
                                            send_frame,
                                            user_data,
                                            ping,
                                        })
                                    }
                                    (MsgType::Reconnect, #queue::#variant_name) =>
                                        self.#reconnect_sender.send(::ilium::server::send::ReconnectSignal {
                                            account,
                                            ping,
                                            send_frame,
                                        }),
                                    (MsgType::Accept, #queue::#variant_name)=>
                                        self.queue.send(::ilium::server::send::QueueSignal::Accept { account }),
                                    (MsgType::Leave, #queue::#variant_name) =>
                                        self.queue.send(::ilium::server::send::QueueSignal::Leave { account }),
                                    (MsgType::Action(action), #queue::#variant_name) =>
                                        self.action.send(::ilium::server::send::ActionSignal { account, action }),
                                )*
                            }?;
                            Ok(())
                        }
                    }
                }
            } else if #[cfg(feature = "client")] {
                quote! { () }
            } else {
                quote! { () }
            }
        }
    };

    let register = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "server")] {
                let user_data: Type = name_value(&ast.attrs, "user_data")
                    .unwrap_or_else(|| abort_call_site!("Could not find user_data attribute"));
                quote! {
                    #(
                        app.register(::ilium::server::matchmaking::process_queue::<#component, #user_data>);
                        app.register(::ilium::server::matchmaking::reconnect::<#component>);
                        app.register(::ilium::server::matchmaking::matchmake::<#component, #user_data>);
                    )*
                }
            } else if #[cfg(feature = "client")] {
                quote! { () }
            } else {
                quote! { () }
            }
        }
    };

    quote! {
        #sender

        impl ::ilium::session::Queue for #queue {
            type Action = #action;
            fn register<A: ::ilium::session::IliumApp>(app: &mut A) {
                #register
            }
            fn insert(&self, ec: &mut ::bevy::ecs::system::EntityCommands) {
                match self {
                    #(Self::#variant_name => ec.insert(#component),)*
                };
            }
        }

        #(
            #[derive(::bevy::prelude::Component)]
            pub struct #component;

            #[derive(::bevy::prelude::Component)]
            pub struct #lobby_name(#lobby_type);

            impl<'a> std::convert::TryFrom<&'a [::bevy::prelude::Entity]> for #lobby_name {
                type Error = std::array::TryFromSliceError;
                fn try_from(v: &[::bevy::prelude::Entity]) -> Result<Self, Self::Error> {
                    let list: #lobby_type = v.try_into()?;
                    Ok(#lobby_name(list))
                }
            }

            impl ::ilium::session::Lobby for #lobby_name {
                fn entities(&self) -> impl Iterator<Item = ::bevy::prelude::Entity> {
                    let Self(list) = self;
                    list.iter().copied()
                }
            }

            impl ::ilium::session::QueueComponent for #component {
                type Queue = #queue;
                type Info = ::ilium::session::Info<
                    <Self::Action as ::ilium::session::Action>::User,
                    <Self::Action as ::ilium::session::Action>::Shared,
                >;
                type Lobby = #lobby_name;
                type Action = #action;
                fn info(
                    index: usize,
                    users: &[<Self::Action as ::ilium::session::Action>::User],
                    shared: &<Self::Action as ::ilium::session::Action>::Shared,
                ) -> Self::Info {
                    ::ilium::session::Info {
                        users: <Self::Action as ::ilium::session::Action>::User::info(index, users, shared),
                        shared: <Self::Action as ::ilium::session::Action>::Shared::info(index, users, shared),
                    }
                }
            }
        )*
    }
    .into()
}
