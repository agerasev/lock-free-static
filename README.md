# lock-free-static

[![Crates.io][crates_badge]][crates]
[![Docs.rs][docs_badge]][docs]
[![Gitlab CI][gitlab_badge]][gitlab]
[![License][license_badge]][license]

[crates_badge]: https://img.shields.io/crates/v/lock-free-static.svg
[docs_badge]: https://docs.rs/lock-free-static/badge.svg
[gitlab_badge]: https://gitlab.com/agerasev/lock-free-static/badges/master/pipeline.svg
[license_badge]: https://img.shields.io/crates/l/lock-free-static.svg

[crates]: https://crates.io/crates/lock-free-static
[docs]: https://docs.rs/lock-free-static
[gitlab]: https://gitlab.com/agerasev/lock-free-static/-/pipelines?scope=branches&ref=master
[license]: #license

Lock-free static variables.

## Examples

### Static cell

```rust
use lock_free_static::OnceCell;

static VAR: OnceCell<i32> = OnceCell::new();

fn main() {
    VAR.set(123).unwrap();
    assert_eq!(*VAR.get().unwrap(), 123);
}
```

### Mutable static cell

```rust
use lock_free_static::OnceMut;

static VAR: OnceMut<i32> = OnceMut::new();

fn main() {
    VAR.set(123).unwrap();

    let mut guard = VAR.lock().unwrap();
    assert_eq!(*guard, 123);
    *guard = 321;
    drop(guard);

    assert_eq!(*VAR.lock().unwrap(), 321);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
