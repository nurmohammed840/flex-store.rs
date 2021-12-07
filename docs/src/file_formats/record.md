# [Record](https://en.wikipedia.org/wiki/Record_(computer_science))

A record is a collection of fields. Records in a relational database are usually called  as "rows". In NOSQL are called "document". 

Example:

```rust
# use std::mem::size_of;
struct Date {
    day: u8,
    month: u8,
    year: u16,
}
println!("{}", size_of::<Date>());
```

This is a fixed-length size record. 