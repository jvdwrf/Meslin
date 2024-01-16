[![Crates.io](https://img.shields.io/crates/v/meslin)](https://crates.io/crates/meslin)
[![Documentation](https://docs.rs/meslin/badge.svg)](https://docs.rs/meslin)

# Meslin
Meslin is a Rust library offering ergonomic wrappers for channels like `mpmc` or `broadcast`. It's designed to ease the creation of actor systems by adding user-friendly features, without being tying the user to any specific runtime. This makes it compatible with various runtimes such as `tokio`, `smol`, or `async-std`. It intentionally steers clear of incorporating supervisory functions or other complex features, focusing instead on simplicity and non-interference.

See the [documentation](https://docs.rs/meslin) for more information.

### License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.