# xrpl-rust
[![latest]][crates.io] [![docs_status]][docs] [![deps_status]][deps] [![audit_status]][audit] [![rustc]][rust] [![unit_status]][unit]

[latest]: https://img.shields.io/crates/v/xrpl.svg
[crates.io]: https://crates.io/crates/xrpl

[docs_status]: https://docs.rs/xrpl/badge.svg
[docs]: https://docs.rs/xrpl

[deps_status]: https://deps.rs/repo/github/589labs/xrpl-rust/status.svg
[deps]: https://deps.rs/repo/github/589labs/xrpl-rust

[audit_status]: https://github.com/589labs/xrpl-rust/actions/workflows/audit_test.yml/badge.svg
[audit]: https://github.com/589labs/xrpl-rust/actions/workflows/audit_test.yml

[rustc]: https://img.shields.io/badge/rust-1.51.0%2B-orange.svg
[rust]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html

[unit_status]: https://github.com/589labs/xrpl-rust/actions/workflows/unit_test.yml/badge.svg
[unit]: https://github.com/589labs/xrpl-rust/actions/workflows/unit_test.yml

A Rust library to interact with the XRPL.

A pure Rust implementation for interacting with the XRP Ledger, the xrpl-rust 
package simplifies the hardest parts of XRP Ledger interaction, like 
serialization and transaction signing, by providing idiomatic Rust 
functionality for XRP Ledger transactions and core server API (rippled) 
objects.

Interactions with this crate occur using data structures from this crate or
core [alloc](https://doc.rust-lang.org/alloc) types with the exception of 
serde for JSON handling and indexmap for dictionaries. The goal is to ensure 
this library can be used on devices without the ability to use a
[std](hhttps://doc.rust-lang.org/std) environment.

> WIP - Help Welcome

# 🕮 Documentation

Documentation is available [here](https://docs.rs/xrpl). TODO

# 🛠 Installation

To install, add the following to your project's `Cargo.toml`:

```toml
[dependencies.xrpl]
version = "0.1.0"
```
# ⚐ Flags

By default, no features are available. 
To use [std](hhttps://doc.rust-lang.org/std) simply enable the flag:

```toml
[dependencies.xrpl]
version = "0.1.0"
features = ["std"]
```

## ‼ Serde

This project uses  [serde](https://serde.rs) for JSON handling.

## ‼ Indexmap

This project uses [indexmap](https://docs.rs/crate/indexmap) as HashMap is 
not supported in the std crate. TODO: Support both.

## ⚙ #![no_std]

This library aims to be `#![no_std]` compliant.
