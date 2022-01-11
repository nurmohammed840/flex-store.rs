This library provides a data view for reading and writing data in a byte array.

This library requires [feature(generic_const_exprs)](https://blog.rust-lang.org/inside-rust/2021/09/06/Splitting-const-generics.html) to be enabled. whice is a nightly feature.

So you need nightly compiler to use this library.

By default, this library uses little endian as the default endian.
But you can override the endian by using `BE` (for big endian) or `NE` (for native endian) in fetures flag.
 
For example, if you want to use big endian,
```toml
data-view = { version = "0.1", features = ["BE"] }
```

It also works with `[no_std]` environment.

# Example

```rust
use data_view::DataView;
 
let mut buf: [u8; 16] = [0; 16];

buf.write::<u16>(0, 42);
buf.write::<u32>(2, 123);

assert_eq!(buf.read::<u16>(0), 42);
assert_eq!(buf.read::<u32>(2), 123);
```