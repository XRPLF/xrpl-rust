# xrpl-rust ![](https://img.shields.io/crates/v/xrpl.svg) [![](https://docs.rs/xrpl/badge.svg)](https://docs.rs/xrpl) [![Unit](https://github.com/sephynox/xrpl-rust/actions/workflows/unit_test.yml/badge.svg)](https://github.com/sephynox/xrpl-rust/actions/workflows/unit_test.yml) ![](https://img.shields.io/badge/License-ISC-blue)

A Rust library to interact with the XRPL.

A pure Rust implementation for interacting with the XRP Ledger, the xrpl-rust package simplifies the hardest parts of XRP Ledger interaction, like serialization and transaction signing, by providing idiomatic Rust functionality for XRP Ledger transactions and core server API (rippled) objects.

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

By default, no features are available. To use `std` simply enable to the
flag:

```toml
[dependencies.xrpl]
version = "0.1.0"
features = ["std"]
```

## ‼ Serde

This project uses  [serde](https://serde.rs) for JSON handling.

## ⚙ #![no_std]

This library aims to be `#![no_std]` compliant.
