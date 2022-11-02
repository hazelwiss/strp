//! Utility library for parsing data from input strings, or stdin if not with the `no_std` feature.
//! Supports no_std contexts, but requires the alloc crate.
//!
//! ```
//! let (left, right) = strp::scan!("add {}, {}");
//! println!("sum: {}", left + right);
//! ```

#![cfg_attr(feature = "no_std", no_std)]
#![warn(missing_docs)]

extern crate alloc;
extern crate self as strp;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod __private {
    pub use macros;

    use crate::TryParseError;
    use core::iter::Peekable;

    #[inline(always)]
    pub fn parse_single<S: ::strp::TryParse>(
        iter: &mut Peekable<impl Iterator<Item = u8> + Clone>,
        m_str: &'static str,
        delim: Option<u8>,
    ) -> Result<S, TryParseError<S::Err>> {
        let cmp = m_str.bytes();
        let iter_err = iter.clone();
        if iter.by_ref().take(cmp.len()).eq(cmp) {
            if let Some(delim) = delim {
                let iter = iter.by_ref();
                let mut iter = core::iter::from_fn(|| iter.next_if(|e| *e != delim));
                S::try_parse(&mut iter)
            } else {
                S::try_parse(iter)
            }
        } else {
            Err(TryParseError::ExpectedMismatch(
                m_str,
                iter_err.map(|b| b as char).collect(),
            ))
        }
    }

    pub trait ParseMultiple: Sized {
        fn parse_multiple(
            iter: &mut Peekable<impl Iterator<Item = u8> + Clone>,
            sparse_data: &[(&'static str, Option<u8>)],
        ) -> Result<Self, ()>;
    }

    impl<T: strp::TryParse, const LEN: usize> ParseMultiple for [T; LEN] {
        #[inline(always)]
        fn parse_multiple(
            iter: &mut Peekable<impl Iterator<Item = u8> + Clone>,
            sparse_data: &[(&'static str, Option<u8>)],
        ) -> Result<Self, ()> {
            assert!(LEN == sparse_data.len());
            let mut array: [T; LEN] = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
            for i in 0..LEN {
                let cur = unsafe { sparse_data.get_unchecked(i) };
                *unsafe { array.get_unchecked_mut(i) } =
                    parse_single(iter, cur.0, cur.1).or(Err(()))?;
            }
            Ok(array)
        }
    }

    macro_rules! impl_sparse_multiple_tuple {
        ($first:ident, $($rest:ident),+; $size:expr;) => {
            impl<$first: ::strp::TryParse, $($rest: ::strp::TryParse),+> ParseMultiple for ($first, $($rest),+) {
                #[inline(always)]
                fn parse_multiple(
                    iter: &mut Peekable<impl Iterator<Item = u8> + Clone>,
                    sparse_data: &[(&'static str, Option<u8>)],
                ) -> Result<Self, ()> {
                    assert!($size == sparse_data.len());
                    Ok(
                        macros::rep!($size[parse_single(iter, sparse_data[#].0, sparse_data[#].1).or(Err(()))?])
                    )
                }
            }
        };
    }

    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K,L,M,N,O,P; 16;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K,L,M,N,O; 15;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K,L,M,N; 14;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K,L,M; 13;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K,L; 12;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I,K; 11;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J,I; 10;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H,J; 9;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G,H; 8;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F,G; 7;);
    impl_sparse_multiple_tuple!(A,B,C,D,E,F; 6;);
    impl_sparse_multiple_tuple!(A,B,C,D,E; 5;);
    impl_sparse_multiple_tuple!(A,B,C,D; 4;);
    impl_sparse_multiple_tuple!(A,B,C; 3;);
    impl_sparse_multiple_tuple!(A,B; 2;);
}

cfg_if::cfg_if! {
    if #[cfg(feature = "no_std")]{
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __readline {
            ($str:literal, $($tt:tt)*) => {
                {
                    #![allow(unreachable_code)]
                    const _: () = panic!("attempting to read from stdin with the `no_std` feature enabled!");
                    unreachable!();
                    $($tt)*!("" => $str)
                }
            };
        }
    } else {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __readline {
            ($str:literal, $($tt:tt)*) => {
                {
                    let mut string = ::std::string::String::new();
                    ::std::io::stdin().read_line(&mut string).unwrap();
                    string.pop();
                    $($tt)*!(string => $str)
                }
            };
        }
    }
}

/// Attempts to parse a single varialbe from an iterator on a type that implements
/// the `TryParse` trait.
///
/// The macro takes in a source expression, which is then matched against a string
/// literal in order to match a single value from the source. The source expression
/// must evaluate into a type that derives the `TryParse`trait.
///
/// ```
/// // The whole source string will be parsed as a u32.
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
/// // An error is returned, since `source` doesn't
/// // match the matching string.
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
/// // try_parse also works well on &str as `source`.
/// let source = "abcd";
/// let v = try_parse!(source => "{}");
/// assert_eq!(v, Ok("abcd".to_string()));
///
/// // uses stdin instead of a source string.
/// let v: f64 = try_parse!("{}");
/// println!("{v}");
/// ```
#[macro_export]
macro_rules! try_parse {
    ($expr:expr => $str:literal) => {
        ::strp::__private::macros::try_parse!($expr => $str)
    };
    ($str:literal) =>  {
        {
            ::strp::__readline!($str, ::strp::try_parse)
        }
    };
}

/// Attempts to parse a single variable from an iterator described by a string literal.
/// Panics on error.
///
/// For more details read the documentation of the `try_parse` macro.
#[macro_export]
macro_rules! parse {
    ($expr:expr => $str:literal) => {
        ::strp::try_parse!($expr => $str).unwrap()
    };
    ($str:literal) =>  {
        {
            ::strp::__readline!($str, ::strp::parse)
        }
    };
}

/// Very similar to `try_parse`, except it allows for 2 or more matched values.
///
/// For more details read the documenation of the `try_parse` macro.
///
/// ```
/// let source = "10, 20, 30, 40";
/// let v = try_scan!(source => "{}, {}, {}, {}");
/// assert_eq!(v, Ok((10, 20, 30, 40)));
///
/// // uses stdin as source.
/// let (l,r): (u32,u32) = try_scan!("add {}, {}").unwrap();
/// println!("{}", l + r);
/// ```
#[macro_export]
macro_rules! try_scan {
    ($expr:expr => $str:literal) => {
        ::strp::__private::macros::try_scan!($expr => $str)
    };
    ($str:literal) =>  {
        {
            ::strp::__readline!($str, ::strp::try_scan)
        }
    };
}

/// Attempts to parse multiple variables from an iterator described by a string literal.
/// Panics on error.
///
/// For more details read the documentation of the `try_scan` macro.
#[macro_export]
macro_rules! scan {
    ($expr:expr => $str:literal) => {
        ::strp::try_scan!($expr => $str).unwrap()
    };
    ($str:literal) =>  {
        {
            ::strp::__readline!($str, ::strp::scan)
        }
    };
}

/// Allows a type to be parsed through `try_parse`, `parse`, `try_scan` and `scan` macros.
pub trait TryParse
where
    Self: Sized,
{
    /// Error type used in the TryParseError.
    type Err;

    /// Attempts to parse the type using an u8 iterator.
    fn try_parse(iter: &mut impl Iterator<Item = u8>) -> Result<Self, TryParseError<Self::Err>>;
}

macro_rules! impl_from_str_tys {
    ($($ty:ty),*) => {
        $(
            impl TryParse for $ty where Self: ::core::str::FromStr {
                type Err = <Self as ::core::str::FromStr>::Err;

                fn try_parse(
                    iter: &mut impl Iterator<Item = u8>,
                ) -> Result<Self, TryParseError<Self::Err>> {
                    Ok(::core::str::FromStr::from_str(
                        core::str::from_utf8(&iter.collect::<alloc::vec::Vec<u8>>()).expect(""),
                    )?)
                }
            }
        )*
    };
}

impl_from_str_tys!(
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    f32,
    f64,
    alloc::string::String
);

/// Generic error type for parsing.
#[derive(PartialEq)]
pub enum TryParseError<T> {
    /// The pattern in the source string doesn't match
    /// the given pattern.
    ExpectedMismatch(&'static str, alloc::string::String),
    /// Contains a generic error from `T`.
    Err(T),
}

impl<T: core::fmt::Debug> core::fmt::Debug for TryParseError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ExpectedMismatch(s, i) => write!(f, "expected: \"{s}\"\ngot: \"{i}\"."),
            Self::Err(arg0) => arg0.fmt(f),
        }
    }
}

impl<T> From<T> for TryParseError<T> {
    fn from(value: T) -> Self {
        Self::Err(value)
    }
}
