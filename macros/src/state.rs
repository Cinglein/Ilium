use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote};
use crate::util::*;
use syn::*;

enum IliumFieldInfo {
    Open { name: Ident, ty: Type },
    Hidden { name: Ident, ty: Type },
    Private { name: Ident, ty: Type },
}

pub fn derive_state_impl(input: TokenStream, is_shared: bool) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let state: Ident = ast.ident;
    let info_name: Ident = format_ident!("{state}Info");
    let other: Type = if is_shared {
        name_value(&ast.attrs, "user").unwrap_or_else(|| abort_call_site!("Could not find state attribute")) 
    } else {
        name_value(&ast.attrs, "shared").unwrap_or_else(|| abort_call_site!("Could not find state attribute")) 
    };
    let init = name_value::<Path>(&ast.attrs, "init")
        .map(|path| if is_shared { quote!(#path(seed)) } else { quote!(#path(shared, users)) })
        .unwrap_or(if is_shared { quote!(Default::default()) } else { quote!(vec![Default::default(); users]) });
    let timers: Vec<Ident> = name_value::<CommaSeparated<Ident>>(&ast.attrs, "timers").map(|t| t.0).unwrap_or_default();
    let mut open_name: Vec<Ident> = Vec::new();
    let mut open_type: Vec<Type> = Vec::new();
    let mut hidden_name: Vec<Ident> = Vec::new();
    let mut hidden_type: Vec<Type> = Vec::new();
    let mut hidden_fn: Vec<Ident> = Vec::new();
    let mut private_name: Vec<Ident> = Vec::new();
    let mut private_type: Vec<Type> = Vec::new();

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            for field in named.iter() {
                let name = field.ident.clone().unwrap();
                let ty = field.ty.clone();
                let info = if name_value::<LitBool>(&field.attrs, "open").is_some_and(|b| b.value) {
                    IliumFieldInfo::Open { ty, name }
                } else if let Some(hidden) = name_value::<HiddenFn>(&field.attrs, "hidden") {
                    hidden_fn.push(hidden.ident);
                    IliumFieldInfo::Hidden { ty: hidden.output, name }
                } else {
                    IliumFieldInfo::Private { ty, name }
                };

                match info {
                    IliumFieldInfo::Open { name, ty } => {
                        open_name.push(name);
                        open_type.push(ty);
                    }
                    IliumFieldInfo::Hidden { name, ty } => {
                        hidden_name.push(name);
                        hidden_type.push(ty);
                    }
                    IliumFieldInfo::Private { name, ty } => {
                        private_name.push(name);
                        private_type.push(ty);
                    }
                }
            }
        }
        _ => abort_call_site!("Only structs are supported."),
    };
    let info = if is_shared {
        quote! {
            #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
            pub struct #info_name {
                #(#open_name: #open_type,)*
                #(#hidden_name: #hidden_type,)*
            }
        }
    } else {
        quote! {
            #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
            pub struct #info_name {
                #(#open_name: #open_type,)*
                #(#hidden_name: #hidden_type,)*
                #(#private_name: Option<#private_type>,)*
            }
        }
    };
    let timer = quote! {
        impl ::ilium::session::AsStopwatch for #state {
            fn pause(&mut self) {
                #(self.#timers.pause();)*
            }
            fn unpause(&mut self) {
                #(self.#timers.unpause();)*
            }
            fn reset(&mut self) {
                #(self.#timers.reset();)*
            }
            fn tick(&mut self, delta: ::std::time::Duration) {
                #(self.#timers.tick(delta);)*
            }
        }
    }; 

    if is_shared {
        quote! {
            #info
            #timer
            
            impl ::ilium::session::SharedState for #state {
                type Info = #info_name;
                type User = #other; 
                fn info<S: ::ilium::session::AsState<Shared = #state>>(index: S::Index, state: &S) -> Self::Info {
                    let shared = state.shared();
                    let shared: &#state = ::std::borrow::Borrow::borrow(&shared);
                    #info_name {
                        #(#open_name: shared.#open_name.clone(),)*
                        #(#hidden_name: #hidden_fn(index, state),)*
                    }
                }
                fn init(seed: [u8; 32]) -> Self {
                    #init
                }
            }
        }
        .into()
    } else {
        quote! {
            #info
            #timer

            impl ::ilium::session::UserState for #state {
                type Info = #info_name;
                type Shared = #other;
                fn info<S: ::ilium::session::AsState<User = #state>>(index: S::Index, state: &S) -> 
                    ::bevy::utils::hashbrown::HashMap<S::Index, Self::Info> {
                    state.users().map(|(i, user)| {
                        let user: &#state = ::std::borrow::Borrow::borrow(&user);
                        let info = #info_name {
                            #(#open_name: user.#open_name.clone(),)*
                            #(#hidden_name: #hidden_fn(i, state),)*
                            #(#private_name: if index == i { Some(user.#private_name.clone()) } else { None },)*
                        };
                        (i, info)
                    })
                    .collect()
                }
                fn init(shared: &mut Self::Shared, users: usize) -> Vec<Self> {
                    #init
                }
            }
        }
        .into()
    }
}
