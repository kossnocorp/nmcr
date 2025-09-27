# Rust Crate

## Lib

Rust crate library template.

### `./Cargo.toml`

```toml
[package]
name = "{{pkg_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
```

### `./src/lib.rs`

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```
