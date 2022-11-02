use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{bracketed, parse::Parse, Expr, LitInt, Token};

struct Sparse {
    input: Expr,
    #[allow(unused)]
    arrow: Token![=>],
    mstr: String,
}

impl Parse for Sparse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse()?;
        let arrow = input.parse()?;
        let string: syn::LitStr = input.parse()?;
        Ok(Self {
            input: expr,
            arrow,
            mstr: string.value(),
        })
    }
}

struct Var {
    #[allow(unused)]
    inlined: Option<Ident>,
}

struct Sensetized {
    m_str: String,
    content: Option<(Var, Option<char>, Option<Box<Sensetized>>)>,
}

fn sensetize_single(iter: &mut Peekable<Chars>) -> Sensetized {
    let mut m_str = String::new();
    let mut var_option = None;
    'outer: while let Some(c) = iter.next() {
        if c == '{' {
            let mut c = iter.next().expect("missing token after '{'");
            if c == '{' {
                m_str.push('{')
            } else {
                let mut ident = String::new();
                loop {
                    match c {
                        '}' => {
                            let inlined = if ident.is_empty() {
                                None
                            } else {
                                Some(Ident::new(&ident, Span::call_site()))
                            };
                            var_option = Some(Var { inlined });
                            break 'outer;
                        }
                        c => ident.push(c),
                    }
                    c = iter.next().expect("missibng closing '}' after '{'")
                }
            }
        } else if c == '}' {
            assert_eq!(
                iter.next(),
                Some('}'),
                "special token '}}' not properly used. Did you mean to write \"}}}}\"?"
            );
            m_str.push('}');
        } else {
            m_str.push(c)
        }
    }
    if let Some(var_option) = var_option {
        let delim_option = iter.peek().copied();
        Sensetized {
            m_str,
            content: Some((var_option, delim_option, None)),
        }
    } else {
        Sensetized {
            m_str,
            content: None,
        }
    }
}

fn sensetize_multiple(iter: &mut Peekable<Chars>) -> Sensetized {
    let mut cur = sensetize_single(iter);
    if iter.peek().is_some() {
        if let Some(content) = &mut cur.content {
            content.2 = Some(Box::new(sensetize_multiple(iter)));
        }
    }
    cur
}

#[proc_macro]
pub fn try_parse(ts: TokenStream) -> TokenStream {
    let sparse: Sparse = syn::parse(ts).expect("string parsing failed");
    let sensetized = sensetize_multiple(&mut sparse.mstr.chars().peekable());
    let source = sparse.input;
    let m_str = sensetized.m_str;
    let (_var, delim, next) = if let Some(content) = sensetized.content {
        (
            content.0,
            {
                if let Some(c) = content.1 {
                    quote!(::core::option::Option::Some(#c as u8))
                } else {
                    quote!(::core::option::Option::None)
                }
            },
            content.2,
        )
    } else {
        panic!("missing \"{{}}\"")
    };
    let next = if let Some(next) = next {
        assert!(
            next.content.is_none(),
            "attempting to parse multiple values during a `parse!`. Try using `scan!` instead."
        );
        let m_str = next.m_str;
        Some(quote! {
            if !iter.clone().eq(#m_str.bytes()){
                let err: ::alloc::string::String = iter.map(|b| b as char).collect();
                Err(::strp::TryParseError::ExpectedMismatch(#m_str, err))?
            }
        })
    } else {
        None
    };
    quote! {
        {(|| {
            let source = &#source;
            let slice = ::core::convert::AsRef::<[u8]>::as_ref(source);
            let mut iter = slice.iter().cloned().peekable();
            let var = ::strp::__private::parse_single(&mut iter, #m_str, #delim);
            #next;
            var
        })()}
    }
    .into()
}

#[proc_macro]
pub fn try_scan(ts: TokenStream) -> TokenStream {
    let sparse: Sparse = syn::parse(ts).expect("string parsing failed");
    let sensetized = sensetize_multiple(&mut sparse.mstr.chars().peekable());
    let mut vars = vec![];
    let mut tail = None;
    let mut cur = Box::new(sensetized);
    loop {
        let m_str = cur.m_str;
        if let Some(content) = cur.content {
            vars.push((content.0, m_str, content.1));
            if let Some(next) = content.2 {
                cur = next;
            } else {
                break;
            }
        } else {
            tail = Some(m_str);
            break;
        }
    }
    let mut quote_slice = vec![];
    for var in vars.into_iter() {
        let m_str = var.1;
        let delim = if let Some(option) = var.2 {
            quote!(::core::option::Option::Some(#option as u8))
        } else {
            quote!(::core::option::Option::None)
        };
        quote_slice.push(quote!((#m_str, #delim)));
    }
    let quote_tail = if let Some(tail) = tail {
        quote! {
            if !iter.clone().eq(#tail.bytes()) {
                let err: ::alloc::string::String = iter.map(|b| b as char).collect();
                Err(::strp::TryParseError::ExpectedMismatch(#tail, err))?
            }
        }
    } else {
        quote!()
    };
    let source = sparse.input;
    quote! {
        {(|| {
            let __v_parse_result: Result<_, ::strp::TryParseError<_>> = {
                let source = &#source;
                let slice = ::core::convert::AsRef::<[u8]>::as_ref(source);
                let mut iter = slice.iter().cloned().peekable();
                let ret = ::strp::__private::ParseMultiple::parse_multiple(&mut iter, &[#(#quote_slice),*]).or_else(|e| Err(::strp::TryParseError::Err(e)));
                #quote_tail;
                ret
            };
            __v_parse_result
        })()}
    }
    .into()
}

struct Rep(String);

impl Parse for Rep {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let count: LitInt = input.parse()?;
        let count = count
            .base10_parse()
            .expect("failed to parse count in repeat macro");
        let content;
        let _brace = bracketed!(content in input);
        let content: proc_macro2::TokenStream = content.parse()?;
        let content = content.to_string();
        let mut string = String::new();
        for i in 0..count {
            string.push_str(&content.clone().replace('#', &i.to_string()));
            string.push(',');
        }
        Ok(Self(string))
    }
}

#[proc_macro]
pub fn rep(ts: TokenStream) -> TokenStream {
    let rep: Rep = syn::parse::<Rep>(ts).expect("expected a valid repeat statement!");
    let new_ts =
        proc_macro2::TokenStream::from_str(&rep.0).expect("failed to turn into a token stream");
    quote!((#new_ts)).into()
}
