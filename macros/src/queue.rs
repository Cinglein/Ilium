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
                lobby_name.push(name);
                lobby_type.push(ty);
            }
        }
        _ => abort_call_site!("Only enums are supported."),
    };

    let register = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "server")] {
                quote! {
                    #(
                        app.register(::ilium::server::process_queue::<#component>);
                        app.register(::ilium::server::reconnect::<#component>);
                        app.register(::ilium::server::matchmake::<#component>);
                    )*
                };
            } else if #[cfg(feature = "client")] {
                quote! { () }
            } else {
                quote! { () }
            }
        }
    };

    let tokens = quote! {
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
    .into();
    eprintln!("{tokens}");
    tokens
}
