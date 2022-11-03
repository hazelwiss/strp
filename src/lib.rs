//! Utility library for parsing data from an input string, or stdin if built with the `std` feature.
//! Supports no_std contexts when built without the `std` feature enabled. Requires the alloc crate.
//! The `std` feature is enabled by default.
//!
//! Supports parsing one or multiple values from a string. Can parse primitives, Strings, or any
//! type which derives the `TryParse` trait.
//!
//! Supports parsing primitives from hexadecimal or binary values.
//!
//! The `try_parse`, `parse`, `try_scan` and `scan` macros put high emphasis on deducing types,
//! meaning you rarely need to specify the type yourself unless you want to enforce a specific
//! type, or there's missing context.
//!
//! # Basic `parse` and `try_parse` usage
//!
//! ```
//! # use strp::{try_parse, parse};
//! // `parse` and `try_parse` parses a single value from the source string,
//! // and has more cohesive errors than `scan` and `try_scan`.
//!
//! // Attempts to parse  a number from `source` using `try_parse`
//! let source = String::from("number: 30");
//! let number = try_parse!(source => "number: {}");
//! assert_eq!(number, Ok(30));
//!
//! // Internally calls `try_parse` and unwraps the result.
//! let source = "hello, world!";
//! let value: String = parse!(source => "hello, {}!");
//! assert_eq!(value, "world".to_string());
//! ```
//!
//! # Basic `scan` and `try_scan` usage
//!
//! ```
//! # use strp::{try_scan, scan};
//! // `scan` and `try_scan` has less cohesive erros than `parse` and
//! // `try_parse`, but allows parsing multiple values from a single
//! // source string.
//!
//! // Example of parsing 4 strings from one source string using `try_scan`
//! let source = String::from("this is four words!");
//! let matched = try_scan!(source => "{} {} {} {}!");
//! assert_eq!(
//!     matched,
//!     Ok((
//!         "this".to_string(),
//!          "is".to_string(),
//!          "four".to_string(),
//!          "words".to_string()
//!     ))
//! );
//!
//! // Interally calls `try_scan` and unwraps the result.
//! let source = "add 20, 30";
//! let (left, right): (u32, u32) = scan!(source => "add {}, {}");
//! assert_eq!(left + right, 50);
//! ```
//!
//! # Using stdin with the `std` feature.
//!
//! ```no_run
//! # use strp::{try_scan, scan, try_parse, parse};
//! let name: String = parse!("hello! my name is {}.");
//! println!("hello, {name}!");
//!
//! let try_parse: Result<String, _> = try_parse!("Please, enter your name: {}.");
//! match try_parse {
//!     Ok(name) => println!("Thank you for inputing your name, {name}!"),
//!     Err(_) => println!("No name was given."),
//! }
//!
//! // You can also use stdin for `scan` and `try_scan`
//! let (a, b, c): (u32, u32, u32) = scan!("{} + {} = {}");
//! assert_eq!(a + b, c);
//!
//! let try_scan: Result<(u32, u32, u32), _> = try_scan!("{} + {} = {}");
//! match try_scan {
//!     Ok((a,b,c)) => println!("{a} + {b} = {c}"),
//!     Err(e) => println!("an erro occured: {e:?}"),
//! }
//! ```
//!
//! # Inlining matched values.
//!
//! ```
//! # use strp::{scan, try_parse, parse, try_scan};
//! let mut number = -1;
//! try_parse!("input number: 20" => "input number: {number}");
//! assert_eq!(number, 20);
//!
//! let (mut l, mut r) = ("".to_string(), "".to_string());
//! try_scan!("hello world!" => "{l} {r}").expect("failed to parse");
//! assert_eq!((l, r), ("hello".to_string(), "world!".to_string()));
//!
//! // If the parsing failed, an error is returned by the macro call.
//! let mut number: i32 = -1;
//! match try_parse!("fail 20" => "success {number}"){
//!     Ok(_) => println!("parsed value: {number}"),
//!     Err(_) => println!("failed to parse input string"),
//! }
//!
//! // Inlining can also be paired with returning values in `scan` and `try_scan`.
//! let (mut left, mut right) = ("".to_string(), "".to_string());
//! let middle = scan!("left middle right" => "{left} {} {right}");
//! assert_eq!(
//!     (left, middle, right),
//!     ("left".to_string(), "middle".to_string(), "right".to_string())
//! );
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
//!
//! # Hexadecimal and binary parsing.
//!
//! ```
//! # use strp::{scan, try_parse, parse, try_scan};
//! // Need to specify 'u64' here, since otherwise the value will be too large.
//! let hex: Result<u64, _> =  
//!     try_parse!("input hex: 0x0123456789ABCDEF" => "input hex: 0x{:x}");
//! assert_eq!(hex, Ok(0x0123456789ABCDEF));
//!
//! let bin: Result<u32, _> = try_parse!("input bin: 0b11110001" => "input bin: 0b{:b}");
//! assert_eq!(bin, Ok(0b11110001));
//!
//! let (bin, hex) = scan!("bin: 0b101, hex: 0xFE" => "bin: 0b{:b}, hex: 0x{:x}");
//! assert_eq!((bin, hex), (0b101u32, 0xFEu32));
//!
//! // Parsing as hexadecimal or binary also works with inlining.
//! let mut bin = -1;
//! parse!("binary value: 101" => "binary value: {bin:b}");
//! assert_eq!(bin, 0b101);
//!
//! let (mut bin, mut hex) = (-1, -1);
//! scan!("bin: 1111, hex: F" => "bin: {bin:b}, hex: {hex:x}");
//! assert_eq!((bin, hex), (0b1111, 0xF));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![allow(clippy::result_unit_err)]

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

    pub struct Hex<T>(T);
    pub struct Binary<T>(T);

    impl<T> Hex<T> {
        #[inline(always)]
        pub fn new(val: T) -> Self {
            Self(val)
        }

        #[inline(always)]
        pub fn into_inner(self) -> T {
            self.0
        }
    }

    impl<T> Binary<T> {
        #[inline(always)]
        pub fn new(val: T) -> Self {
            Self(val)
        }

        #[inline(always)]
        pub fn into_inner(self) -> T {
            self.0
        }
    }

    macro_rules! impl_hex__and_binary_for_primitives {
        ($($ty:ty),*) => {
            $(
                impl crate::TryParse for Hex<$ty> {
                    type Err = core::num::ParseIntError;

                    fn try_parse(
                        iter: &mut impl Iterator<Item = u8>,
                    ) -> Result<Self, TryParseError<Self::Err>> {
                        let vec = iter.collect::<alloc::vec::Vec<u8>>();
                        let str = core::str::from_utf8(&vec)
                            .or(Err(TryParseError::InvalidUtf8String))?;
                        Ok(Self(<$ty>::from_str_radix(&str, 16)?))
                    }
                }

                impl From<Hex<$ty>> for $ty{
                    fn from(hex: Hex<$ty>) -> Self {
                        hex.0
                    }
                }

                impl crate::TryParse for Binary<$ty> {
                    type Err = core::num::ParseIntError;

                    fn try_parse(
                        iter: &mut impl Iterator<Item = u8>,
                    ) -> Result<Self, TryParseError<Self::Err>> {
                        let vec = iter.collect::<alloc::vec::Vec<u8>>();
                        let str = core::str::from_utf8(&vec)
                            .or(Err(TryParseError::InvalidUtf8String))?;
                        Ok(Self(<$ty>::from_str_radix(&str, 2)?))
                    }
                }

                impl From<Binary<$ty>> for $ty{
                    fn from(bin: Binary<$ty>) -> Self {
                        bin.0
                    }
                }
            )*

        };
    }

    impl_hex__and_binary_for_primitives!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

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
                    parse_single::<T>(iter, cur.0, cur.1).or(Err(()))?;
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
                        core::str::from_utf8(&iter.collect::<__private::alloc::vec::Vec<u8>>()).or(Err(TryParseError::InvalidUtf8String))?,
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
    /// The input string was given as invalid utf8.
    InvalidUtf8String,
    /// Contains a generic error from `T`.
    Err(T),
}

impl<T: core::fmt::Debug> core::fmt::Debug for TryParseError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ExpectedMismatch(s, i) => write!(f, "expected: \"{s}\"\ngot: \"{i}\"."),
            Self::InvalidUtf8String => write!(f, "invalid utf8 encoding in source string."),
            Self::Err(arg0) => arg0.fmt(f),
        }
    }
}

impl<T> From<T> for TryParseError<T> {
    fn from(value: T) -> Self {
        Self::Err(value)
    }
}
