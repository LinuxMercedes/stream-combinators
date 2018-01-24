# stream-combinators

Additional stream combinators for [`futures-rs`](https://github.com/alexcrichton/futures-rs/) streams:

- `filter_fold` accumulates one stream into another.
- `sequence` transforms a source stream into a new stream immediately before the source stream produces a value.

See the source files and examples for much more detailed documentation and motivating examples.

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
stream-combinators = "0.1.0"
```

Next, add this to your crate:

```rust
extern crate stream_combinators;
```

To use `filter_fold`, include the following:

```rust
use stream_combinators::FilterFoldStream;
```

Or to use `sequence`, include the following:

```rust
use stream_combinators::SequenceStream;
```

These combinators can be used on any `Stream` in the same way that
[the default combinators](https://docs.rs/futures/0.1/futures/stream/trait.Stream.html) are used.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Futures by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
