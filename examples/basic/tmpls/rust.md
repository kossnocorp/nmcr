# Rust

## Package Cargo.toml

Crate manifest file.

`Cargo.toml`:

```rust
[package]
name = "nmcr_{{name}}"
version = "0.1.0"
edition = "2024"
description = "{{description}}"
authors = ["Sasha Koss <koss@nocorp.me>"]
license = "MIT"
repository = "https://github.com/kossnocorp/nmcr"
```

## Package .gitignore

Rust crate .gitignore file.

`.gitignore`:

```
# Rust
/target/
# Temp
tmp/
```
