# strp

Utility library for parsing data from an input string, or stdin if built with the `std` feature.
Supports no_std contexts without the `std` feature, but requires the alloc crate.
The `std` feature is enabled by default.

Supports parsing one or multiple values from a string. Can parse primitives, Strings, or any
type which derives the `TryParse` trait.

Supports parsing primitives from hexadecimal or binary values.

The macros put high emphasis on deducing types, meaning you rarely need to specify the type yourself
unless you want to enforce a specific type, or there's missing context.

## Basic `parse` and `try_parse` usage

```rust
// `parse` and `try_parse` parses a single value from the source string,
// and has more cohesive errors than `scan` and `try_scan`.

// Attempts to parse  a number from `source` using `try_parse`
let source = String::from("number: 30");
let number = try_parse!(source => "number: {}");
assert_eq!(number, Ok(30));

// Internally calls `try_parse` and unwraps the result.
let source = "hello, world!";
let value: String = parse!(source => "hello, {}!");
assert_eq!(value, "world".to_string());
```

## Basic `scan` and `try_scan` usage

```rust
// `scan` and `try_scan` has less cohesive erros than `parse` and
// `try_parse`, but allows parsing multiple values from a single
// source string.

// Example of parsing 4 strings from one source string using `try_scan`
let source = String::from("this is four words!");
let matched = try_scan!(source => "{} {} {} {}!");
assert_eq!(
    matched,
    Ok((
        "this".to_string(),
         "is".to_string(),
         "four".to_string(),
         "words".to_string()
    ))
);

// Interally calls `try_scan` and unwraps the result.
let source = "add 20, 30";
let (left, right): (u32, u32) = scan!(source => "add {}, {}");
assert_eq!(left + right, 50);
```

## Using stdin with the `std` feature.

```rust
let name: String = parse!("hello! my name is {}.");
println!("hello, {name}!");

let try_parse: Result<String, _> = try_parse!("Please, enter your name: {}.");
match try_parse {
    Ok(name) => println!("Thank you for inputing your name, {name}!"),
    Err(_) => println!("No name was given."),
}

// You can also use stdin for `scan` and `try_scan`
let (a, b, c): (u32, u32, u32) = scan!("{} + {} = {}");
assert_eq!(a + b, c);

let try_scan: Result<(u32, u32, u32), _> = try_scan!("{} + {} = {}");
match try_scan {
    Ok((a,b,c)) => println!("{a} + {b} = {c}"),
    Err(e) => println!("an erro occured: {e:?}"),
}
```

## Inlining matched values.

```rust
let mut number = -1;
try_parse!("input number: 20" => "input number: {number}");
assert_eq!(number, 20);

let (mut l, mut r) = ("".to_string(), "".to_string());
try_scan!("hello world!" => "{l} {r}").expect("failed to parse");
assert_eq!((l, r), ("hello".to_string(), "world!".to_string()));

// Inlining can also be paired with returning values in `scan` and `try_scan`.
let (mut left, mut right) = ("".to_string(), "".to_string());
let middle = scan!("left middle right" => "{left} {} {right}");
assert_eq!(
    (left, middle, right),
    ("left".to_string(), "middle".to_string(), "right".to_string())
);

// `scan` and `try_scan` can mix both inlining matching values,
// or alternatively capture them as a return value.
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

let (mut x, mut y) = (0, 0);
let v = try_scan!("10, 10, 20, 20" => "{x}, {}, {y}, {}");
assert_eq!(v, Ok((x,y)));
```

## Hexadecimal and binary parsing.

```rust
let hex: Result<u64, _> /* Need to specify 'u64' here, since otherwise the value will be too large. */ =
    try_parse!("input hex: 0x0123456789ABCDEF" => "input hex: 0x{:x}");
assert_eq!(hex, Ok(0x0123456789ABCDEF));

let bin: Result<u32, _> = try_parse!("input bin: 0b11110001" => "input bin: 0b{:b}");
assert_eq!(bin, Ok(0b11110001));

let (bin, hex) = scan!("bin: 0b101, hex: 0xFE" => "bin: 0b{:b}, hex: 0x{:x}");
assert_eq!((bin, hex), (0b101u32, 0xFEu32));

// Parsing as hexadecimal or binary also works with inlining.
let mut bin = -1;
parse!("binary value: 101" => "binary value: {bin:b}");
assert_eq!(bin, 0b101);

let (mut bin, mut hex) = (-1, -1);
scan!("bin: 1111, hex: F" => "bin: {bin:b}, hex: {hex:x}");
assert_eq!((bin, hex), (0b1111, 0xF));
```

License: MIT
