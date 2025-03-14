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
    let timer: Option<Ident> = name_value(&ast.attrs, "timer");
    let stopwatch: Option<Ident> = name_value(&ast.attrs, "stopwatch");
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
                let info = if name_value::<LitBool>(&field.attrs, "open").map_or(false, |b| b.value) {
                    println!("name: {name:?}");
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
    let info = quote! {
        #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #info_name {
            #(#open_name: #open_type,)*
            #(#hidden_name: #hidden_type,)*
            #(#private_name: Option<#private_type>,)*
        }
    };
    let timer = if let Some(timer) = timer {
        quote! {
            impl ::ilium::session::AsStopwatch for #state {
                fn elapsed_secs(&self) -> f32 {
                    self.#timer.elapsed_secs()
                }
                fn pause(&mut self) {
                    self.#timer.pause();
                }
                fn unpause(&mut self) {
                    self.#timer.unpause();
                }
                fn is_paused(&self) -> bool {
                    self.#timer.is_paused()
                }
                fn reset(&mut self) {
                    self.#timer.reset();
                }
                fn tick(&mut self, delta: Duration) {
                    self.#timer.tick(delta);
                }
            }

            impl ::ilium::session::AsTimer for #state {
                fn remaining_secs(&self) -> f32 {
                    self.#timer.remaining_secs()
                }
                fn finished(&self) -> bool {
                    self.#timer.finished()
                }
            }
        } 
    } else {
        quote! {}
    };
    let stopwatch = if let Some(stopwatch) = stopwatch {
        quote! {
            impl ::ilium::session::AsStopwatch for #state {
                fn elapsed_secs(&self) -> f32 {
                    self.#stopwatch.elapsed_secs()
                }
                fn pause(&mut self) {
                    self.#stopwatch.pause();
                }
                fn unpause(&mut self) {
                    self.#stopwatch.unpause();
                }
                fn is_paused(&self) -> bool {
                    self.#stopwatch.is_paused()
                }
                fn reset(&mut self) {
                    self.#stopwatch.reset();
                }
                fn tick(&mut self, delta: Duration) {
                    self.#stopwatch.tick(delta);
                }
            }
        } 
    } else {
        quote! {}
    };

    if is_shared {
        quote! {
            #info
            #timer
            #stopwatch
            
            impl ::ilium::session::SharedState for #state {
                type Info = #info_name;
                type User = #other; 
                fn info<S: ::ilium::session::AsState<Shared = #state>>(index: S::Index, state: &S) -> Self::Info {
                    let shared = state.shared();
                    let shared: &#state = ::std::borrow::Borrow::borrow(&shared);
                    #info_name {
                        #(#open_name: shared.#open_name.clone(),)*
                        #(#hidden_name: #hidden_fn(index, state),)*
                        #(#private_name: None,)*
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
            #stopwatch

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
