# babysub-rust

An educational SAT preprocessor written in Rust.

# Running

```
cargo run -- [OPTIONS] [CNF PATH] [OUT PATH]
```

# Testing

Since simplification is not implemented now, all tests will fail.

```
cargo test
```

# Logging

```
cargo run --features "logging" -- [OPTIONS] [CNF PATH] [OUT PATH]
```
