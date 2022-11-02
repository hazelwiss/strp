# strp

Utility library for parsing data from input strings, or stdin if not with the `no_std` feature.
Supports no_std contexts, but requires the alloc crate.

```rust
let (left, right) = strp::scan!("add {}, {}");
println!("sum: {}", left + right);
```
