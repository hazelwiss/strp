# strp

Utility library for parsing data from input strings, or stdin if not built with the `no_std` feature.
Supports no_std contexts, but requires the alloc crate.

```rust
// `scan` parses two or more values from an input string.
// Panics on failure.
let (left, right): (u32, u32) = scan!("add {}, {}");
println!("sum: {}", left + right);

// `parse` parses a single value from a string, but has more
// cohesive errors.
// Panics on failure.
let value: String = parse!("hello, {}!");
println!("your name is: {value}");

// You can also attempt parsing, returning an Err on failure.
// The `scan` equivalent is `try_scan`.
let value: Result<u32, _> = try_parse!("write here: {}");
match value{
    Ok(value) => println!("input that was written there: {value}"),
    Err(e) => println!("failed to parse the input string! {e:?}"),
}

// Matched values may also be inlined into the match string.
let number;
try_parse!("input_number: 20" => "input number: {number}");
assert_eq!(number, Ok(20));

let (mut l, mut r) = ("".to_string(), "".to_string());
try_scan!("hello world!" => "{l} {r}").expect("failed to parse");
assert_eq!((l, r), ("hello".to_string(), "world!".to_string()));

// `scan` and `try_scan` can mix both inlining mathing values,
// or capture them as a return value.
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

License: MIT
