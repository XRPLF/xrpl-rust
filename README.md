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
[std](https://doc.rust-lang.org/std) environment.

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

By default, the following features are enabled:

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

# Command Line Interface

The XRPL Rust library provides a powerful CLI tool for interacting with the XRP Ledger directly from your terminal. This makes it easy to perform common XRPL operations without writing code.

## Installation

To install the CLI tool, you can add the `cli` feature to your dependencies:

```toml
[dependencies.xrpl]
version = "0.5.0"
features = ["cli"]
```

Or install it directly using Cargo:

```bash
cargo install xrpl-rust --features=cli
```

## Basic Usage

After installation, you can use the CLI with the `xrpl` command:

```bash
xrpl [COMMAND] [OPTIONS]
```

For help with available commands:

```bash
xrpl --help
```

For help with a specific command:

```bash
xrpl [COMMAND] --help
```

## Available Commands

The CLI offers commands in several categories:

### Wallet Commands

| Command  | Subcommand  | Description                                     |
| -------- | ----------- | ----------------------------------------------- |
| `wallet` | `generate`  | Generate a new XRPL wallet                      |
| `wallet` | `from-seed` | Create a wallet from an existing seed           |
| `wallet` | `faucet`    | Generate a wallet funded by the testnet faucet  |
| `wallet` | `validate`  | Validate an XRPL address (classic or X-address) |

### Account Commands

| Command   | Subcommand   | Description                                     |
| --------- | ------------ | ----------------------------------------------- |
| `account` | `info`       | Get basic account information                   |
| `account` | `tx`         | Get account transactions                        |
| `account` | `objects`    | Get account objects (trust lines, offers, etc.) |
| `account` | `channels`   | Get account payment channels                    |
| `account` | `currencies` | Get currencies an account can send/receive      |
| `account` | `lines`      | Get account trust lines                         |

### Transaction Commands

| Command       | Subcommand | Description                                |
| ------------- | ---------- | ------------------------------------------ |
| `transaction` | `sign`     | Sign a transaction using your seed         |
| `transaction` | `submit`   | Submit a signed transaction to the network |

### Server and Ledger Commands

| Command  | Subcommand  | Description                              |
| -------- | ----------- | ---------------------------------------- |
| `server` | `fee`       | Get the current network fee              |
| `server` | `info`      | Get information about a rippled server   |
| `ledger` | `data`      | Get data about a specific ledger         |
| `server` | `subscribe` | Subscribe to ledger events via WebSocket |

## Command Details

### Wallet Operations

#### wallet generate

Generate a new XRPL wallet (keypair).

```bash
# Generate a new wallet
xrpl wallet generate

# Generate and save wallet (functionality not yet implemented)
xrpl wallet generate --save
```

#### wallet from-seed

Derive a wallet from an existing seed.

```bash
xrpl wallet from-seed --seed s123... [--sequence 0]
```

Parameters:

- `--seed`: The seed to use (required)
- `--sequence`: The key sequence number (default: 0)

#### wallet faucet

Generate a new wallet and fund it using the testnet faucet.

```bash
xrpl wallet faucet [--url https://s.altnet.rippletest.net:51234]
```

Parameters:

- `--url`: The testnet URL (default: https://s.altnet.rippletest.net:51234)

#### wallet validate

Validate an XRPL address (works with both classic and X-addresses).

```bash
xrpl wallet validate --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
```

Parameters:

- `--address`: The address to validate (required)

### Account Information

#### account info

Get basic account information.

```bash
xrpl account info --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account tx

Get account transactions.

```bash
xrpl account tx --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--limit 10] [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--limit`: Maximum number of transactions to return (default: 10)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account objects

Get account objects (trust lines, offers, etc.)

```bash
xrpl account objects --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--type-filter offer] [--limit 10] [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--type-filter`: Type of objects to return (e.g., "offer", "state")
- `--limit`: Maximum number of objects to return (default: 10)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account channels

Get information about an account's payment channels.

```bash
xrpl account channels --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--destination-account rDestination...] [--limit 10] [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--destination-account`: Filter channels by destination account
- `--limit`: Maximum number of channels to return (default: 10)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account currencies

Get a list of currencies that an account can send or receive.

```bash
xrpl account currencies --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account lines

Get information about an account's trust lines.

```bash
xrpl account lines --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh [--peer rPeer...] [--limit 10] [--url https://xrplcluster.com/]
```

Parameters:

- `--address`: The account address (required)
- `--peer`: Filter trust lines by peer account
- `--limit`: Maximum number of trust lines to return (default: 10)
- `--url`: The XRPL node URL (default: https://xrplcluster.com/)

#### account set-flag

Set an account flag.

```bash
xrpl account set-flag --seed s... --flag asfRequireAuth [--url https://xrplcluster.com/]
```

Parameters:

- `--seed`, `-s` (required): The seed to use for signing
- `--flag`, `-f` (required): The flag to set (e.g., asfRequireAuth, asfDisableMaster, etc.)
- `--url`, `-u` (optional, default: https://xrplcluster.com/): The XRPL node URL

**Example Output:**
```text
Signed transaction blob: ...
To submit, use: xrpl transaction submit --tx-blob ... --url ...
```

#### account clear-flag

Clear an account flag.

```bash
xrpl account clear-flag --seed s... --flag asfRequireAuth [--url https://xrplcluster.com/]
```

Parameters:

- `--seed`, `-s` (required): The seed to use for signing
- `--flag`, `-f` (required): The flag to clear (e.g., asfRequireAuth, asfDisableMaster, etc.)
- `--url`, `-u` (optional, default: https://xrplcluster.com/): The XRPL node URL

**Example Output:**
```text
Signed transaction blob: ...
To submit, use: xrpl transaction submit --tx-blob ... --url ...
```
