use syn::{parse::*, punctuated::Punctuated, *};

const NAME: &str = "ilium";

pub struct HiddenFn {
    pub ident: Ident,
    _arrow: Token![->],
    pub output: Type,
}

impl Parse for HiddenFn {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(HiddenFn {
            ident: input.parse()?,
            _arrow: input.parse()?,
            output: input.parse()?,
        })
    }
}

pub struct CommaSeparated<T: Parse>(pub Vec<T>);

impl<T: Parse> Parse for CommaSeparated<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let punctuated = Punctuated::<T, Token![,]>::parse_terminated(input)?;
        let list = punctuated.into_iter().collect();
        Ok(Self(list))
    }
}

impl<T: Parse> IntoIterator for CommaSeparated<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug)]
enum AttrInner {
    Path(Path),
    NameValue(Box<MetaNameValue>),
}

impl Parse for AttrInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let Ok(eq_token) = input.parse() else {
            return Ok(AttrInner::Path(path));
        };
        let Ok(value) = input.parse() else {
            return Ok(AttrInner::Path(path));
        };
        Ok(AttrInner::NameValue(Box::new(MetaNameValue {
            path,
            eq_token,
            value,
        })))
    }
}

impl AttrInner {
    fn matches_path(&self, p: &str) -> bool {
        let p = parse_str::<Path>(p).expect("Invalid path provided");
        match self {
            Self::Path(path) => path == &p,
            Self::NameValue(namevalue) => namevalue.path == p,
        }
    }
}

pub fn name_value<T: Parse>(attrs: &[Attribute], p: &str) -> Option<T> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(MetaList { path, tokens, .. })
            if path.get_ident().is_some_and(|ident| *ident == NAME) =>
        {
            let parser = Punctuated::<AttrInner, Token![,]>::parse_terminated;
            let punctuated = parser.parse2(tokens.clone()).ok()?;
            punctuated.iter().find_map(|meta| {
                meta.matches_path(p)
                    .then(|| match meta {
                        AttrInner::Path(_) => parse_str::<T>("true").ok(),
                        AttrInner::NameValue(namevalue) => match &namevalue.value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(name),
                                ..
                            }) => name.parse().ok(),
                            _ => None,
                        },
                    })
                    .flatten()
            })
        }
        _ => None,
    })
}
