use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
#[warn(missing_docs)]
use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};
use syn::{bracketed, parse::Parse, Expr, LitInt, LitStr, Token};

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

#[doc(hidden)]
#[proc_macro]
pub fn try_parse_proc(ts: TokenStream) -> TokenStream {
    let sparse: Sparse = syn::parse(ts).expect("string parsing failed");
    let sensetized = sensetize_multiple(&mut sparse.mstr.chars().peekable());
    let source = sparse.input;
    let m_str = sensetized.m_str;
    let (var, delim, next) = if let Some(content) = sensetized.content {
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
    let var_ident = Ident::new("var", Span::mixed_site());
    let ret = if let Some(next) = next {
        assert!(
            next.content.is_none(),
            "attempting to parse multiple values during a `parse!`. Try using `scan!` instead."
        );
        let m_str = next.m_str;
        quote! {
            if iter.clone().eq(#m_str.bytes()){
                #var_ident
            } else{
                let err: ::strp::__private::alloc::string::String = iter.map(|b| b as char).collect();
                Err(::strp::TryParseError::ExpectedMismatch(#m_str, err))
            }
        }
    } else {
        quote!(#var_ident)
    };
    let result_ident = Ident::new("__parse_result", Span::mixed_site());
    let assign_or_ret = if let Some(inlined) = var.inlined {
        quote! {
            #inlined = #result_ident;
            ()
        }
    } else {
        quote!(#result_ident)
    };
    quote! {
        {
            extern crate alloc;
            let #result_ident = (|| {
                let source = &#source;
                let slice = ::core::convert::AsRef::<[u8]>::as_ref(source);
                let mut iter = slice.iter().cloned().peekable();
                let #var_ident = ::strp::__private::parse_single(&mut iter, #m_str, #delim);
                #ret
            })();
            #assign_or_ret
        }
    }
    .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn try_scan_proc(ts: TokenStream) -> TokenStream {
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
    let result_ident = Ident::new("__v_parse_result", Span::mixed_site());
    let mut quote_slice = vec![];
    let mut ret_tuple = vec![];
    let mut ret_assign = vec![];
    let var_count = vars.len();
    for (i, var) in vars.into_iter().enumerate() {
        let index = LitInt::new(&i.to_string(), Span::call_site());
        if let Some(inlined) = var.0.inlined {
            ret_assign.push(quote!(#inlined = #result_ident.#index))
        } else {
            ret_tuple.push(quote!(#result_ident.#index))
        }
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
                let err: ::strp::__private::alloc::string::String = iter.map(|b| b as char).collect();
                return Err(::strp::TryParseError::ExpectedMismatch(#tail, err));
            }
        }
    } else {
        quote!()
    };
    let source = sparse.input;
    let type_quote = {
        let inner = Vec::from_iter((0..var_count).map(|_| quote!(_)));
        quote!(( #(#inner),* ))
    };
    quote! {
        {(|| {
            extern crate alloc;
            let #result_ident: Result<#type_quote, ::strp::TryParseError<_>> = (|| {
                let source = &#source;
                let slice = ::core::convert::AsRef::<[u8]>::as_ref(source);
                let mut iter = slice.iter().cloned().peekable();
                match ::strp::__private::ParseMultiple::parse_multiple(&mut iter, &[#(#quote_slice),*]){
                    Ok(ok) => {
                        #quote_tail;
                        Ok(ok)
                    }
                    Err(_) => Err(::strp::TryParseError::Err(())),
                }
            })();
            match #result_ident{
                Err(e) => Err(e),
                Ok(#result_ident) => {
                    #(#ret_assign;)*
                    Ok((#(#ret_tuple),*))
                }
            }

        })()}
    }
    .into()
}

enum MacroInput {
    Source(Expr, String),
    Stdin(String),
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if let Ok(str) = input.parse::<LitStr>() {
            if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                let expr = Expr::Lit(syn::ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Str(LitStr::new(&str.value(), Span::call_site())),
                });
                let m_str: LitStr = input.parse()?;
                Self::Source(expr, m_str.value())
            } else {
                Self::Stdin(str.value())
            }
        } else {
            let expr = input.parse::<Expr>().expect("expected expression");
            input.parse::<Token![=>]>()?;
            let m_str = input
                .parse::<LitStr>()
                .expect("expected string literal after '=>'");
            Self::Source(expr, m_str.value())
        })
    }
}

macro_rules! __impl__ {
    ($ts:ident, $err:literal, $($tt:tt)*) => {
        {
            let input = syn::parse::<MacroInput>($ts).expect($err);
            match input {
                MacroInput::Source(expr, literal) => {
                    quote!({ $($tt)*!(#expr => #literal) })
                }
                MacroInput::Stdin(literal) => {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "std")]{
                            quote!{
                                {
                                    let mut string = ::std::string::String::new();
                                    ::std::io::stdin().read_line(&mut string).unwrap();
                                    string.pop();
                                    $($tt)*!(string => #literal)
                                }
                            }
                        } else{
                            quote!{
                                {
                                    const _: () = panic!("attempting to read from stdin with the `std` feature disabled!");
                                    unreachable!();
                                    $($tt)*!("" => #literal)
                                }
                            }
                        }
                    }
                }
            }
        }
    };
}

/// Attempts to parse a single variable from an iterator on a type that implements
/// the `TryParse` trait.
///
/// Accepts a source expression, which is then matched against a string
/// literal in order to match a single value from the source. The source expression
/// must evaluate into a type that derives the `TryParse`trait.
///
/// ```no_run
/// # use crate::{scan, try_scan, parse, try_parse};
///
/// // The whole source string will be parsed into a u32.
/// let source = "20".to_string();
/// let v = try_parse!(source => "{}");
/// assert_eq!(v, Ok(20u32));
///
/// // Only "world" will be parsed into `v`, since the rest
/// // of the `source` string won't be matched into a value.
/// let source = "hello world!".to_string();
/// let v = try_parse!(source => "hello {}!");
/// assert_eq!(v, Ok("world".to_string()));
///
/// // An error is returned, since `source` doesn't match the
/// // matching string.
/// let source = "abcd".to_string();
/// let v: Result<String, _> = try_parse!(source => "123{}");
/// assert!(matches!(v, Err(_)));
///
/// // `source` does match the matching string, but fails to
/// // parse 'd' as a u32.
/// let source = "123d".to_string();
/// let v: Result<u32, _> = try_parse!(source => "123{}");
/// assert!(matches!(v, Err(_)));
///
/// // `try_parse` also works well on a non-literal as `source`.
/// let source = "abcd";
/// let v = try_parse!(source => "{}");
/// assert_eq!(v, Ok("abcd".to_string()));
///
/// // Uses stdin instead of a source string.
/// // Only available with the `std` feature.
/// let v: f64 = parse!("{}");
/// println!("{v}");
///
/// // Inlines the matched value. This causes `parse` to return ()
/// // and instead assigns the parse result to v directly.
/// let v: u32;
/// parse!("u32:{v}");
/// println!("{v}");
/// ```
#[proc_macro]
pub fn try_parse(ts: TokenStream) -> TokenStream {
    __impl__!(
        ts,
        "invalid input for `try_parse`:",
        ::strp::__private::macros::try_parse_proc
    )
    .into()
}

/// Attempts to parse a single variable from an iterator described by a string literal.
/// Panics on error.
///
/// For more details read the documentation of the `try_parse` macro.
#[proc_macro]
pub fn parse(ts: TokenStream) -> TokenStream {
    let ts = __impl__!(
        ts,
        "invalid input for `parse`:",
        ::strp::__private::macros::try_parse_proc
    );
    quote!(#ts.unwrap()).into()
}

/// Very similar to `try_parse`, except it allows for 2 or more matched values.
///
/// For more details read the documenation of the `try_parse` macro.
///
/// ```no_run
/// # use strp::try_scan;
///
/// let source = "10, 20, 30, 40";
/// let v = strp::try_scan!(source => "{}, {}, {}, {}");
/// assert_eq!(v, Ok((10, 20, 30, 40)));
///
/// // Uses stdin as source.
/// let (l,r): (u32,u32) = strp::try_scan!("add {}, {}").unwrap();
/// println!("{}", l + r);
/// ```
#[proc_macro]
pub fn try_scan(ts: TokenStream) -> TokenStream {
    __impl__!(
        ts,
        "invalid input for `try_scan`:",
        ::strp::__private::macros::try_scan_proc
    )
    .into()
}

/// Attempts to parse multiple variables from an iterator described by a string literal.
/// Panics on error.
///
/// For more details read the documentation of the `try_scan` macro.
#[proc_macro]
pub fn scan(ts: TokenStream) -> TokenStream {
    let ts = __impl__!(
        ts,
        "invalid input for `scan`:",
        ::strp::__private::macros::try_scan_proc
    );
    quote!(#ts.unwrap()).into()
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
