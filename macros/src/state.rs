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
                let info = field
                    .attrs
                    .iter()
                    .find_map(|attr| match &attr.meta {
                        Meta::List(MetaList { path, tokens, .. })
                            if path
                                .get_ident()
                                .is_some_and(|ident| &ident.to_string() == "ilium") =>
                        {
                            match parse2(tokens.clone()).expect("Could not parse info attribute") {
                                Meta::Path(path)
                                    if path
                                        .get_ident()
                                        .is_some_and(|id| &id.to_string() == "open") =>
                                {
                                    let ty = ty.clone();
                                    let name = name.clone();
                                    Some(IliumFieldInfo::Open { ty, name })
                                }
                                Meta::NameValue(MetaNameValue {
                                    path,
                                    value:
                                        Expr::Lit(ExprLit {
                                            lit: Lit::Str(name),
                                            ..
                                        }),
                                    ..
                                }) if path
                                    .get_ident()
                                    .is_some_and(|ident| &ident.to_string() == "hidden") =>
                                {
                                    hidden_fn
                                        .push(name.parse().expect("Could not parse hidden fn"));
                                    let ty = ty.clone();
                                    let name = parse_quote!("::ilium::session::Hidden<#ty>");
                                    Some(IliumFieldInfo::Hidden { ty, name })
                                }
                                Meta::Path(path)
                                    if path
                                        .get_ident()
                                        .is_some_and(|id| &id.to_string() == "private") =>
                                {
                                    let ty = ty.clone();
                                    let ty = ty.clone();
                                    let name = name.clone();
                                    Some(IliumFieldInfo::Private { ty, name })
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or(IliumFieldInfo::Private { ty, name });
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

    if is_shared {
        quote! {
            #info
            
            impl ::ilium::session::SharedState for #state {
                type Info = #info_name;
                type User = #other; 
                fn info(index: usize, users: &[Self::User], shared: &Self) -> Self::Info {
                    #info_name {
                        #(#open_name: user.#open_name,)*
                        #(#hidden_name: if #hidden_fn(index, users, shared) {
                            ::ilium::session::Hidden::Seen(user.#hidden_name)
                        } else {
                            ::ilium::session::Hidden::Unseen
                        },)*
                        #(#private_name: None,)*
                    }
                }
            }
        }
        .into()
    } else {
        quote! {
            #info

            impl ::ilium::session::UserState for #state {
                type Info = #info_name;
                type Shared = #other;
                fn info(index: usize, users: &[Self], shared: &Self::Shared) -> Vec<Self::Info> {
                    users.iter().enumerate().map(|(i, user)| {
                        #info_name {
                            #(#open_name: user.#open_name.clone(),)*
                            #(#hidden_name: if #hidden_fn(index, users, shared) { 
                                ::ilium::session::Hidden::Seen(user.#hidden_name.clone())
                            } else {
                                ::ilium::session::Hidden::Unseen
                            },)*
                            #(#private_name: if index == i { Some(user.#private_name.clone()) } else { None })*
                        }
                    })
                    .collect()
                }
            }
        }
        .into()
    }
}
