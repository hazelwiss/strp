//! Utility library for parsing data from input strings, or stdin if built with the `std` feature.
//! Supports no_std contexts, but requires the alloc crate.
//!
//! ```no_run
//! # use strp::{scan, try_parse, parse, try_scan};
//! // `scan` parses two or more values from stdin.
//! // Panics on failure.
//! let (left, right): (u32, u32) = scan!("add {}, {}");
//! println!("sum: {}", left + right);
//!
//! // `parse` parses a single value from a string, but has more
//! // cohesive errors.
//! // Panics on failure.
//! let value: String = parse!("hello, {}!");
//! println!("your name is: {value}");
//!
//! // You can also attempt parsing, returning an Err on failure.
//! // The `scan` equivalent is `try_scan`.
//! let value: Result<u32, _> = try_parse!("write here: {}");
//! match value{
//!     Ok(value) => println!("input that was written there: {value}"),
//!     Err(e) => println!("failed to parse the input string! {e:?}"),
//! }
//!
//! // Uses a source string rather than piping stdin.
//! let value: String = try_parse!("this string is matched" => "{} string is matched");
//! assert_eq!(value, Ok("this".to_string()));
//!
//! // Matched values may also be inlined into the match string.
//! let number;
//! try_parse!("input_number: 20" => "input number: {number}");
//! assert_eq!(number, Ok(20));
//!
//! let (mut l, mut r) = ("".to_string(), "".to_string());
//! try_scan!("hello world!" => "{l} {r}").expect("failed to parse");
//! assert_eq!((l, r), ("hello".to_string(), "world!".to_string()));
//!
//! // `scan` and `try_scan` can mix both inlining matching values,
//! // or alternatively capture them as a return value.
//! let (mut x, mut y, mut z) = (0, 0, 0);
//! let v = try_scan!("10, 20, 30, 40" => "{}, {x}, {y}, {z}");
//! assert_eq!((v, x, y, z), (Ok(10), 20, 30, 40));
//!
//! let (mut x, mut y, mut z) = (0, 0, 0);
//! let v = try_scan!("10, 20, 30, 40" => "{x}, {}, {y}, {z}");
//! assert_eq!((v, x, y, z), (Ok(20), 10, 30, 40));
//!
//! let (mut x, mut y, mut z) = (0, 0, 0);
//! let v = try_scan!("10, 20, 30, 40" => "{x}, {y}, {}, {z}");
//! assert_eq!((v, x, y, z), (Ok(30), 10, 20, 40));
//!
//! let (mut x, mut y, mut z) = (0, 0, 0);
//! let v = try_scan!("10, 20, 30, 40" => "{x}, {y}, {z}, {}");
//! assert_eq!((v, x, y, z), (Ok(40), 10, 20, 30));
//!
//! let (mut x, mut y) = (0, 0);
//! let v = try_scan!("10, 10, 20, 20" => "{x}, {}, {y}, {}");
//! assert_eq!(v, Ok((x,y)));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[doc(hidden)]
pub extern crate strp_macros as macros;

extern crate self as strp;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod __private {
    pub extern crate alloc;
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

pub use macros::{parse, scan, try_parse, try_scan};

/// Allows a type to be parsed through the `try_parse`, `parse`, `try_scan` and `scan` macros.
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
                    iter: &mut impl  core::iter::Iterator<Item = u8>,
                ) -> Result<Self, TryParseError<Self::Err>> {
                    Ok(::core::str::FromStr::from_str(
                        core::str::from_utf8(&iter.collect::<__private::alloc::vec::Vec<u8>>()).expect(""),
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
    __private::alloc::string::String
);

/// Generic error type for parsing.
#[derive(PartialEq)]
pub enum TryParseError<T> {
    /// The pattern in the source string doesn't match
    /// the given pattern.
    ExpectedMismatch(&'static str, __private::alloc::string::String),
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
