# xrpl-rust ![Downloads](https://img.shields.io/crates/d/xrpl-rust)

[![latest]][crates.io] [![deps_status]][deps] [![audit_status]][audit] [![unit_status]][unit]

[latest]: https://img.shields.io/crates/v/xrpl-rust.svg
[crates.io]: https://crates.io/crates/xrpl-rust
[docs_status]: https://docs.rs/xrpl-rust/badge.svg
[docs]: https://docs.rs/xrpl-rust/latest/xrpl/
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

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="/assets/xrpl-rust_white.png">
  <img alt="" src="/assets/xrpl-rust_black.png">
</picture>

> [!WARNING]
> This repository is under active development. All releases before v1.0.0 can be considered as **beta-releases**

A Rust library to interact with the XRPL.
Based off of the [xrpl-py](https://github.com/XRPLF/xrpl-py) library.

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
version = "0.5.0"
```

# 🕮 Documentation [![docs_status]][docs]

Documentation is available [here](https://docs.rs/xrpl-rust).

## ⛮ Quickstart

TODO - Most core functionality is in place and working.

In Progress:

- no_std examples
- Benchmarks

# ⚐ Flags

By default, the the following features are enabled:

- std
- core
- models
- wallet
- utils
- websocket
- json-rpc
- helpers
- tokio-rt

When `helpers` is enabled you also need to specify a `*-rt` feature flag as it is needed for waiting between requests when using the `submit_and_wait` function.

To operate in a `#![no_std]` environment simply disable the defaults
and enable features manually:

```toml
[dependencies.xrpl]
version = "*"
default-features = false
features = ["core", "models", "wallet", "utils", "websocket", "json-rpc", "helpers", "embassy-rt"]
```

## ⚙ #![no_std]

This library aims to be `#![no_std]` compliant.

# 🕊 Contributing [![contributors_status]][contributors]

If you want to contribute to this project, see [CONTRIBUTING](CONTRIBUTING.md).

# 🗎 License [![license_status]][license]

The `xrpl-rust` library is licensed under the ISC License.
See [LICENSE](LICENSE) for more information.
