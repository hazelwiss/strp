extern crate std;

use crate::{try_parse, try_scan};
use std::string::{String, ToString};

#[test]
fn parse_single() {
    // Test some general cases that should always pass.

    let number = try_parse!("number: 30" => "number: {}");
    assert_eq!(number, Ok(30u32));

    let v = try_parse!("20" => "{}");
    assert_eq!(v, Ok(20));

    let v = try_parse!("20, 30" => "{}, 30");
    assert_eq!(v, Ok(20));

    let v = try_parse!("hell31o, world!" => "hell{}o, world!");
    assert_eq!(v, Ok(31));

    let v = try_parse!("{30}" => "{{{}}}");
    assert_eq!(v, Ok(30));

    let v = try_parse!("hello world!" => "hello {}");
    assert_eq!(v, Ok("world!".to_string()));

    let v = try_parse!("hello world!" => "hello world!{}");
    assert_eq!(v, Ok("".to_string()));

    let v = try_parse!("hello world!" => "hello world{}!");
    assert_eq!(v, Ok("".to_string()));

    for str in ["10", "20", "30"] {
        let v = try_parse!(str => "{}");
        assert_eq!(v, Ok(str.to_string()));
    }

    for number in [10, 20, 30] {
        let v = try_parse!(number.to_string() => "{}");
        assert_eq!(v, Ok(number));
    }

    // Test some special cases.

    // Assures success when matching a souce string between contraints.
    let v = try_parse!("left middle right" => "left {} right");
    assert_eq!(v, Ok("middle".to_string()));

    // Assures success when matching a source string to the left.
    let v = try_parse!("left middle right" => "{} middle right");
    assert_eq!(v, Ok("left".to_string()));

    // Assures success when matching a source string to the right.
    let v = try_parse!("left middle right" => "left middle {}");
    assert_eq!(v, Ok("right".to_string()));

    // Assures an error when the source string doesn't match at the end.
    let v: Result<String, _> = try_parse!("hello world" => "hello world{}!");
    assert!(matches!(v, Err(_)));

    // Assures an error when the source string doesn't match at the start.
    let v: Result<String, _> = try_parse!("worldstr!" => "hello world{}!");
    assert!(matches!(v, Err(_)));

    // Assures an error if the source string is empty.
    let v: Result<u32, _> = try_parse!("" => "hello {}");
    assert!(matches!(v, Err(_)));
}

#[test]
fn parse_multiple() {
    // Test some general cases that should always pass.

    let v = try_scan!("20 30" => "{} {}");
    assert_eq!(v, Ok((20, 30)));

    let v = try_scan!("hello50, worl70d!" => "hello{}, worl{}d!");
    assert_eq!(v, Ok((50, 70)));

    let v = try_scan!("1,2,3,4,5,6,7,8,9" => "1,2,3,{},5,6,7,{},{}");
    assert_eq!(v, Ok((4, 8, 9)));

    let v = try_scan!("hello world!" => "{} {}");
    assert_eq!(v, Ok(("hello".to_string(), "world!".to_string())));

    let v = try_scan!("hello world!" => "he{}llo world{}!");
    assert_eq!(v, Ok(("".to_string(), "".to_string())));

    let v = try_scan!("1,2,3,4,5,6,7,8,9,10" => "{},{},{},{},{},{},{},{},{},{}");
    assert_eq!(v, Ok((1, 2, 3, 4, 5, 6, 7, 8, 9, 10)));

    let v = try_scan!("1,2,3,4,5,6,7,8,9,10" => "{},{},{},{},{},{}");
    assert_eq!(v, Ok((1, 2, 3, 4, 5, "6,7,8,9,10".to_string())));

    let v = try_scan!("this is four words!" => "{} {} {} {}!");
    assert_eq!(
        v,
        Ok((
            "this".to_string(),
            "is".to_string(),
            "four".to_string(),
            "words".to_string()
        ))
    );

    // Test some special cases.

    // Assures an error when the value being scanned does't match at the start.
    let v: Result<(u32, u32), _> = try_scan!("hello world20,30!" => "world{},{}!");
    assert!(matches!(v, Err(_)));

    // Assures an error when the value being scanned doesn't match the end.
    let v: Result<(u32, u32), _> = try_scan!("10 20 40" => "10 20 40 {} {}");
    assert!(matches!(v, Err(_)));

    // Assures an error  when the value being scanned has a trailing whitespace.
    let v: Result<(u32, u32), _> = try_scan!("10 20 30 40 " => "10 {} {} 40");
    assert!(matches!(v, Err(_)));

    // Assures an error when the value being scanned has a mismatch in the middle of parsing.
    let v: Result<(u32, u32, u32, u32), _> = try_scan!("10, 20, 30,, 40 " => "{}, {}, {}, {}");
    assert!(matches!(v, Err(_)));
}

#[allow(unused_must_use)]
#[test]
fn parse_single_inline() {
    // Test some general cases that should always pass.

    let mut v = -1;
    try_parse!("20" => "{v}");
    assert_eq!(v, 20);

    let mut v = -1;
    try_parse!("20, 30" => "{v}, 30");
    assert_eq!(v, 20);

    let mut v = -1;
    try_parse!("hell31o, world!" => "hell{v}o, world!");
    assert_eq!(v, 31);

    let mut v = -1;
    try_parse!("{30}" => "{{{v}}}");
    assert_eq!(v, 30);

    let mut v = "_".to_string();
    try_parse!("hello world!" => "hello {v}");
    assert_eq!(v, "world!".to_string());

    let mut v = "_".to_string();
    try_parse!("hello world!" => "hello world!{v}");
    assert_eq!(v, "".to_string());

    let mut v = "_".to_string();
    try_parse!("hello world!" => "hello world{v}!");
    assert_eq!(v, "".to_string());

    for str in ["10", "20", "30"] {
        let mut v = "_".to_string();
        try_parse!(str => "{v}");
        assert_eq!(v, str.to_string());
    }

    for number in [10, 20, 30] {
        let mut v = -1;
        try_parse!(number.to_string() => "{v}");
        assert_eq!(v, number);
    }

    // Test some special cases.

    // Assures an error when the source string doesn't match at the end.
    let mut v = "_".to_string();
    let res = try_parse!("hello world" => "hello world{v}!");
    assert!(matches!(res, Err(_)));

    // Assures an error when the source string doesn't match at the start.
    let mut v = "_".to_string();
    let res = try_parse!("worldstr!" => "hello world{v}!");
    assert!(matches!(res, Err(_)));

    // Assures an error when the source string is empty and does not match.
    let mut v = -1;
    let res = try_parse!("" => "hello {v}");
    assert!(matches!(res, Err(_)));

    // Assures an error when the source string doesn't match at the end or start.
    let mut v = -1;
    let res = try_parse!("hello, world" => "hello world{v}!");
    assert!(matches!(res, Err(_)));
}

#[test]
#[allow(unused_must_use)]
fn parse_multiple_inlined_mix() {
    // Test some general cases that should always pass.

    let (mut v, mut x) = (0, 0);
    try_scan!("20, 30" => "{v}, {x}");
    assert_eq!((v, x), (20, 30));

    let (mut x, mut y, mut z) = (0, 0, 0);
    let v = try_scan!("10, 20, 30, 40" => "{}, {x}, {y}, {z}");
    assert_eq!((v, x, y, z), (Ok(10), 20, 30, 40));

    let (mut x, mut y, mut z) = (0, 0, 0);
    let v = try_scan!("10, 20, 30, 40" => "{x}, {}, {y}, {z}");
    assert_eq!((v, x, y, z), (Ok(20), 10, 30, 40));

    let (mut x, mut y, mut z) = (0, 0, 0);
    let v = try_scan!("10, 20, 30, 40" => "{x}, {y}, {}, {z}");
    assert_eq!((v, x, y, z), (Ok(30), 10, 20, 40));

    let (mut x, mut y, mut z) = (0, 0, 0);
    let v = try_scan!("10, 20, 30, 40" => "{x}, {y}, {z}, {}");
    assert_eq!((v, x, y, z), (Ok(40), 10, 20, 30));

    let (mut l, mut r) = ("".to_string(), "".to_string());
    try_scan!("hello world!" => "{l}{r}");
    assert_eq!(l, "hello world!");

    let (mut l, mut r) = ("".to_string(), "".to_string());
    try_scan!("hello world!" => "{l} {r}");
    assert_eq!((l, r), ("hello".to_string(), "world!".to_string()));

    let (mut x, mut y) = (0, 0);
    let v = try_scan!("10, 10, 20, 20" => "{x}, {}, {y}, {}");
    assert_eq!(v, Ok((x, y)));
}

#[test]
fn parse_single_special() {
    // Test some general cases that should always pass.

    let v = try_parse!("0xFFFF" => "0x{:x}");
    assert_eq!(v, Ok(0xffff));

    let v = try_parse!("0xabcdefABCDEF" => "0x{:x}");
    assert_eq!(v, Ok(0xabcdef_abcdefu64));

    let v = try_parse!("F0234" => "{:x}");
    assert_eq!(v, Ok(0xf0234));

    let v = try_parse!("0123456789" => "{:x}");
    assert_eq!(v, Ok(0x0123456789u64));

    let v = try_parse!("0xABCDEF" => "0x{:x}");
    assert_eq!(v, Ok(0xabcdef));

    let v = try_parse!("0xabcdef" => "0x{:x}");
    assert_eq!(v, Ok(0xabcdef));

    let v = try_parse!("0b0000" => "0b{:b}");
    assert_eq!(v, Ok(0));

    let v = try_parse!("0b1011" => "0b{:b}");
    assert_eq!(v, Ok(0b1011));

    // Test more special cases.

    // Assures an error when invalid digits are used when parsing binary.
    let v: Result<u32, _> = try_parse!("0b2222" => "0b{:b}");
    assert!(matches!(v, Err(_)));

    // Assures an error when no valid symbol is used when parsing binary.
    let v: Result<u32, _> = try_parse!("0bFFFF" => "0b{:b}");
    assert!(matches!(v, Err(_)));

    // Assures an error when many invalid no valid symbol is used when parsing hexadecimal.
    let v: Result<u64, _> = try_parse!("0xGHJKLMNOPQ" => "0x{:x}");
    assert!(matches!(v, Err(_)));

    // Assures an error when an invalid symbol is used when parsing hexadecimal.
    let v: Result<u64, _> = try_parse!("0xABCDEFG" => "0x{:x}");
    assert!(matches!(v, Err(_)));

    // Assures an error if the number is too large to be parsed.
    let v: Result<u8, _> = try_parse!("0xABCDEFG" => "0x{:x}");
    assert!(matches!(v, Err(_)));
}

#[test]
fn parse_multiple_special() {
    // Test some general cases that should always pass.

    let v = try_scan!("0b0001 + 0xE = 0xF" => "0b{:b} + 0x{:x} = 0x{:x}");
    assert_eq!(v, Ok((0b0001, 0xE, 0xF)));

    // Test more special cases.

    // Assuress success on two successful parsings.
    let v = try_scan!("0b000 + 0x000" => "0b{:b} + 0x{:x}");
    assert_eq!(v, Ok((0, 0)));

    // Assures an error on two failures to parse.
    let v: Result<(u32, u32), _> = try_scan!("0b1234 0xDEFG" => "0b{:b} 0x{:x}");
    assert!(matches!(v, Err(_)));

    // Assures an error on one binary failure to parse.
    let v: Result<(u32, u32), _> = try_scan!("0b1234 0xCDEF" => "0b{:b} 0x{:x}");
    assert!(matches!(v, Err(_)));

    // Assures an error on one hex failure to parse.
    let v: Result<(u32, u32), _> = try_scan!("0b01010 0xDEFG" => "0b{:b} 0x{:x}");
    assert!(matches!(v, Err(_)));
}

#[allow(unused_must_use)]
#[test]
fn parse_single_special_inline() {
    // Test more special cases.

    // Assures success on inlining to parse hex.
    let mut hex: u32 = 0;
    try_parse!("hex: 0xFABC" => "hex: 0x{hex:x}");
    assert_eq!(hex, 0xFABC);

    // Assures success on inlining to parse binary.
    let mut bin: u32 = 0;
    try_parse!("bin: 0b1011" => "bin: 0b{bin:b}");
    assert_eq!(bin, 0b1011);
}

#[allow(unused_must_use)]
#[test]
fn parse_multiple_special_inline() {
    // Test some general cases that should always pass.

    let (mut v0, mut v1, mut v2) = (-1, -1, -1);
    try_scan!("0b0001 + 0xE = 0xF" => "0b{v0:b} + 0x{v1:x} = 0x{v2:x}");
    assert_eq!((v0, v1, v2), (0b0001, 0xE, 0xF));

    // Test more special cases.

    // Assuress success on two successful parsings.
    let (mut v0, mut v1) = (-1, -1);
    try_scan!("0b000 + 0x000" => "0b{v0:b} + 0x{v1:x}");
    assert_eq!((v0, v1), (0, 0));

    // Assures an error on two failures to parse.
    let (mut v0, mut v1) = (-1, -1);
    let res = try_scan!("0b1234 0xDEFG" => "0b{v0:b} 0x{v1:x}");
    assert!(matches!(res, Err(_)));
    assert_eq!((v0, v1), (-1, -1));

    // Assures an error on one binary failure to parse.
    let (mut v0, mut v1) = (-1, -1);
    let res = try_scan!("0b1234 0xCDEF" => "0b{v0:b} 0x{v1:x}");
    assert!(matches!(res, Err(_)));
    assert_eq!((v0, v1), (-1, -1));

    // Assures an error on one hex failure to parse.
    let (mut v0, mut v1) = (-1, -1);
    let res = try_scan!("0b01010 0xDEFG" => "0b{v0:b} 0x{v1:x}");
    assert!(matches!(res, Err(_)));
}

#[allow(unused_must_use)]
#[test]
fn parse_multiple_special_inline_mixed() {
    // Test more special cases.

    // Assures there can be one inlined matched hex value, and one
    // returned matched binary value.
    let mut hex = 0;
    let bin = try_scan!("0b11111 0xFE" => "0b{:b} 0x{hex:x}");
    assert_eq!(bin, Ok(0b11111));
    assert_eq!(hex, 0xFE);

    // Assures there can be one inlined mached binary value, and one
    // return matched hex value.
    let mut bin = 0;
    let hex = try_scan!("0b11111 0xFE" => "0b{bin:b} 0x{:x}");
    assert_eq!(hex, Ok(0xFE));
    assert_eq!(bin, 0b11111);
}
