use syn::{parse::Parse, *};

const NAME: &str = "ilium";

pub fn name_value<T: Parse>(attrs: &[Attribute], p: &str) -> Option<T> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(MetaList { path, tokens, .. })
            if path.get_ident().is_some_and(|ident| *ident == NAME) =>
        {
            match parse2(tokens.clone()) {
                Ok(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(name),
                            ..
                        }),
                    ..
                }) if path == parse_str(p).expect("Invalid path provided") => name.parse().ok(),
                _ => None,
            }
        }
        _ => None,
    })
}
