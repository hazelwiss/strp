extern crate std;

use crate::{try_parse, try_scan};
use std::string::{String, ToString};

#[test]
fn parse_single() {
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

    let v: Result<String, _> = try_parse!("hello world" => "hello world{}!");
    assert!(matches!(v, Err(_)));

    let v: Result<String, _> = try_parse!("worldstr!" => "hello world{}!");
    assert!(matches!(v, Err(_)));

    let v: Result<u32, _> = try_parse!("" => "hello {}");
    assert!(matches!(v, Err(_)));

    let v: Result<u32, _> = try_parse!("hello, world" => "hello, world{}!");
    assert!(matches!(v, Err(_)));
}

#[test]
fn parse_multiple() {
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

    let v: Result<(u32, u32), _> = try_scan!("hello world20,30!" => "world{},{}!");
    assert!(matches!(v, Err(_)));

    let v: Result<(u32, u32), _> = try_scan!("10 20 30" => "10 20 40 {} {}");
    assert!(matches!(v, Err(_)));

    let v: Result<(u32, u32), _> = try_scan!("10 20 30" => "10 {} {} 40");
    assert!(matches!(v, Err(_)));
}

#[test]
fn parse_single_inline() {
    let v;
    try_parse!("20" => "{v}");
    assert_eq!(v, Ok(20));

    let v;
    try_parse!("20, 30" => "{v}, 30");
    assert_eq!(v, Ok(20));

    let v;
    try_parse!("hell31o, world!" => "hell{v}o, world!");
    assert_eq!(v, Ok(31));

    let v;
    try_parse!("{30}" => "{{{v}}}");
    assert_eq!(v, Ok(30));

    let v;
    try_parse!("hello world!" => "hello {v}");
    assert_eq!(v, Ok("world!".to_string()));

    let v;
    try_parse!("hello world!" => "hello world!{v}");
    assert_eq!(v, Ok("".to_string()));

    let v;
    try_parse!("hello world!" => "hello world{v}!");
    assert_eq!(v, Ok("".to_string()));

    for str in ["10", "20", "30"] {
        let v;
        try_parse!(str => "{v}");
        assert_eq!(v, Ok(str.to_string()));
    }

    for number in [10, 20, 30] {
        let v;
        try_parse!(number.to_string() => "{v}");
        assert_eq!(v, Ok(number));
    }

    let v: Result<String, _>;
    try_parse!("hello world" => "hello world{v}!");
    assert!(matches!(v, Err(_)));

    let v: Result<String, _>;
    try_parse!("worldstr!" => "hello world{v}!");
    assert!(matches!(v, Err(_)));

    let v: Result<u32, _>;
    try_parse!("" => "hello {v}");
    assert!(matches!(v, Err(_)));

    let v: Result<u32, _>;
    try_parse!("hello, world" => "hello, world{v}!");
    assert!(matches!(v, Err(_)));
}

#[test]
#[allow(unused_must_use)]
fn parse_multiple_inlined_mix() {
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
