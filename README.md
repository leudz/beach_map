# BeachMap

A BeachMap is just a SlotMap, a data structure used to store elements and access them with an id.

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![Crates.io](https://img.shields.io/crates/v/beach_map.svg)](https://crates.io/crates/beach_map)
[![Documentation](https://docs.rs/beach_map/badge.svg)](https://docs.rs/beach_map)

## Example:
```rust
use beach_map::BeachMap;

let mut beach = BeachMap::default();
let id1 = beach.insert(1);
let id2 = beach.insert(2);

assert_eq!(beach.len(), 2);
assert_eq!(beach[id1], 1);

assert_eq!(beach.remove(id2), Some(2));
assert_eq!(beach.get(id2), None);
assert_eq!(beach.len(), 1);

beach[id1] = 7;
assert_eq!(beach[id1], 7);

beach.extend(0..4);

assert_eq!(beach.data(), [7, 1, 2, 3]);
```
# Rayon
To use rayon with beach_map, you need rayon in your dependencies and add the parallel feature to beach_map.
## Example:
```rust
use beach_map::BeachMap;
use rayon::prelude::*;

let mut beach = BeachMap::default();
let ids = beach.extend(0..500);

beach.par_iter_mut().for_each(|x| {
    *x *= 2;
});

for i in 0..ids.len() {
    assert_eq!(beach[ids[i]], i * 2);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
