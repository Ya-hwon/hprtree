# hprtree

[![Crate](https://img.shields.io/crates/v/hprtree.svg)](https://crates.io/crates/hprtree)
[![API](https://docs.rs/hprtree/badge.svg)](https://docs.rs/hprtree)

## About

This is a Hilbert-Packed-R-Tree implementation for rust (maybe see [Wikipedia](https://en.wikipedia.org/wiki/Hilbert_R-tree)).

The (C++) code that handles the mapping between coordinates and hilbert index was not written by me and can be found on [GitHub](https://github.com/rawrunprotected/hilbert_curves) along with links to interesting writeups on [https://threadlocalmutex.com](https://threadlocalmutex.com).

## Example usage

```rust
use hprtree::{Point, BBox, HPRTreeWrappingBuilder};

let mut index = HPRTreeWrappingBuilder::new(10);
index.insert("Bob", Point{ x: 0f32, y: 0f32 });
for _ in 0..2 {
    index.insert("Alice", Point{ x: 1f32, y: 1f32 });
}
index.insert("James", Point{ x: 2.5f32, y: -2.5f32 });
index.insert("Annie", Point{ x: 20f32, y: 1f32 });
for _ in 0..5 {
    index.insert("Thomas", Point{ x: 1f32, y: -50f32 });
}

let index = index.build();

let mut result = Vec::with_capacity(4);
index.query_with_list(&BBox
           {
               minx: -5f32,
               miny: -5f32,
               maxx: 5f32,
               maxy: 5f32
           }, &mut result);

assert!(result.len() == 4); // this Vec now contains the &strs "Bob", "Alice"(x2) and "James"

for i in result {
    assert!(i == "Bob" || i == "Alice" || i == "James");
    // there are absolutely no guarantees regarding ordering though
}
```

There are more examples in [hprtree.rs](./src/hprtree.rs), [hprtree_wrapping.rs](./src/hprtree_wrapping.rs) (where the example above is from) and in [_examples/](./_examples/)