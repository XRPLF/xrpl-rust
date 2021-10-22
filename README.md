# xrpl-rust ![Downloads](https://img.shields.io/crates/d/xrpl)
[![latest]][crates.io] [![deps_status]][deps] [![audit_status]][audit] [![unit_status]][unit]

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

[contributors]: https://github.com/589labs/xrpl-rust/graphs/contributors
[contributors_status]: https://img.shields.io/github/contributors/589labs/xrpl-rust.svg

[license]: https://opensource.org/licenses/ISC
[license_status]: https://img.shields.io/badge/License-ISC-blue.svg

A Rust library to interact with the XRPL.

A pure Rust implementation for interacting with the XRP Ledger. The xrpl-rust 
crate simplifies the hardest parts of XRP Ledger interaction including
serialization and transaction signing while providing idiomatic Rust 
functionality for XRP Ledger transactions and core server API (rippled) 
objects.

Interactions with this crate occur using data structures from this crate or
core [alloc](https://doc.rust-lang.org/alloc) types with the exception of 
serde for JSON handling and indexmap for dictionaries. The goal is to ensure 
this library can be used on devices without the ability to use a
[std](hhttps://doc.rust-lang.org/std) environment.

> WIP - Help Welcome

# 🛠 Installation [![rustc]][rust]

To install, add the following to your project's `Cargo.toml`:

```toml
[dependencies.xrpl]
version = "0.1.0"
```

# 🕮 Documentation [![docs_status]][docs]

Documentation is available [here](https://docs.rs/xrpl). 

## ⛮ Quickstart
TODO

# ⚐ Flags

By default, the `std` and `core` features are enabled. 
To operate in a `#![no_std]` environment simply disable the defaults
and enable features manually:

```toml
[dependencies.xrpl]
version = "0.1.0"
default-features = false
features = ["core", "models"]
```

## ‼ Serde

This project uses  [serde](https://serde.rs) for JSON handling.

## ‼ Indexmap

This project uses [indexmap](https://docs.rs/crate/indexmap) as HashMap is 
not supported in the alloc crate. TODO: Support both.

## ⚙ #![no_std]

This library aims to be `#![no_std]` compliant.

# 🕊 Contributing [![contributors_status]][contributors]

If you want to contribute to this project, see [CONTRIBUTING.md].

# 🗎 License [![license_status]][license]

The `xrpl-rust` library is licensed under the ISC License. 
See [LICENSE] for more information.
