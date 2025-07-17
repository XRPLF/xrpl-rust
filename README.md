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

# Table of Contents

- [Installation](#installation)
- [Documentation](#documentation)
- [Quickstart](#quickstart)
- [Feature Flags](#feature-flags)
- [no_std Support](#no_std)
- [Command Line Interface](#command-line-interface)
  - [Installation](#installation-1)
  - [Basic Usage](#basic-usage)
  - [Wallet Commands](#wallet-commands)
  - [Account Commands](#account-commands)
  - [Transaction Commands](#transaction-commands)
  - [Server and Ledger Commands](#server-and-ledger-commands)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

# 🛠 Installation [![rustc]][rust]

To install, add the following to your project's `Cargo.toml`:

```toml
[dependencies.xrpl]
version = "0.5.0"
```

# Documentation [![docs_status]][docs]

Documentation is available [here](https://docs.rs/xrpl-rust).

# Quickstart

## Basic Wallet Operations

```rust
use xrpl::wallet::Wallet;

// Generate a new wallet
let wallet = Wallet::create(None)?;
println!("Address: {}", wallet.classic_address);

// Create wallet from seed
let wallet = Wallet::from_seed("sEdV19BLfeQeKdEXyYA4NhjPJe6XBfG", None, false)?;
```

## Making Requests

```rust
use xrpl::clients::XRPLSyncClient;
use xrpl::models::requests::account_info::AccountInfo;

let client = XRPLSyncClient::new("https://xrplcluster.com/")?;
let req = AccountInfo::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(), None, None, None);
let response = client.request(req.into())?;
```

# Feature Flags

## Default Features

- `std` - Standard library support
- `core` - Core XRPL functionality
- `models` - XRPL data models
- `wallet` - Wallet operations
- `utils` - Utility functions
- `websocket` - WebSocket client
- `json-rpc` - JSON-RPC client
- `helpers` - Helper functions (requires runtime)
- `tokio-rt` - Tokio async runtime

## Optional Features

- `cli` - Command line interface
- `embassy-rt` - Embassy async runtime (for no_std)
- `serde` - Serialization support

## Runtime Requirements

When using `helpers`, you must specify a runtime:

- `tokio-rt` - For std environments
- `embassy-rt` - For no_std environments

## #![no_std]

This library aims to be `#![no_std]` compliant.

## `no_std` Usage

```toml
[dependencies.xrpl]
version = "*"
default-features = false
features = ["core", "models", "wallet", "utils", "websocket", "json-rpc", "helpers", "embassy-rt"]
```

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
| `transaction` | `get`      | Get transaction details by hash            |
| `transaction` | `nft-mint` | Create and sign an NFT mint transaction    |
| `transaction` | `nft-burn` | Create and sign an NFT burn transaction    |
| `transaction` | `payment`  | Create and sign a payment transaction      |

### Server and Ledger Commands

| Command  | Subcommand  | Description                              |
| -------- | ----------- | ---------------------------------------- |
| `server` | `fee`       | Get the current network fee              |
| `server` | `info`      | Get information about a rippled server   |
| `ledger` | `data`      | Get data about a specific ledger         |
| `server` | `subscribe` | Subscribe to ledger events via WebSocket |

### 2. **Advanced Query Commands**

| Command       | Subcommand | Description                          |
| ------------- | ---------- | ------------------------------------ |
| `ledger`      | `entry`    | Get a specific ledger entry by index |
| `transaction` | `get`      | Get transaction details by hash      |

#### ledger entry

Get a specific ledger entry by its index.

```bash
xrpl ledger entry --index 1A2B3C... --url https://xrplcluster.com/
```

### 3. **Account NFTs Command**

#### account nfts

Get NFTs owned by an account.

```bash
xrpl account nfts --address rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh --url https://xrplcluster.com/
```

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

#### wallet generate --mnemonic

Generate a new wallet with a BIP39 mnemonic phrase.

```bash
# Generate with 12 words (default)
xrpl wallet generate --mnemonic

# Generate with 24 words
xrpl wallet generate --mnemonic --words 24
```

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

# Library Usage

## Basic Wallet Operations

```rust
use xrpl::wallet::Wallet;

// Generate a new wallet
let wallet = Wallet::create(None)?;
println!("Address: {}", wallet.classic_address);
println!("Seed: {}", wallet.seed);

// Create wallet from seed
let seed = "sEdV19BLfeQeKdEXyYA4NhjPJe6XBfG";
let wallet = Wallet::from_seed(seed, None, false)?;
println!("Classic Address: {}", wallet.classic_address);
```

### Making API Requests

```rust
use xrpl::clients::XRPLSyncClient;
use xrpl::models::requests::{
    account_info::AccountInfo,
    account_lines::AccountLines,
    book_offers::BookOffers,
    ledger::Ledger,
};
use xrpl::models::{LedgerIndex, Currency};

let client = XRPLSyncClient::new("https://xrplcluster.com/")?;

// Get account information
let account_info_req = AccountInfo::new(
    "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
    None, None, None
);
let response = client.request(account_info_req.into())?;

// Get account trust lines
let account_lines_req = AccountLines::new(
    None,
    "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
    None, None, Some(10), None
);
let lines_response = client.request(account_lines_req.into())?;

// Get order book offers
let taker_gets = Currency::xrp();
let taker_pays = Currency::issued("USD", "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
let book_offers_req = BookOffers::new(
    None, taker_gets, taker_pays, None, None, Some(5), None
);
let offers_response = client.request(book_offers_req.into())?;
```

### Working with Transactions (New Builder Pattern)

```rust
use xrpl::models::transactions::{Payment, AccountSet, AccountDelete, CommonFields};
use xrpl::models::{Amount, TransactionType};
use xrpl::wallet::Wallet;
use xrpl::clients::XRPLSyncClient;

// Create a simple XRP payment using the new builder pattern
let payment = Payment {
    common_fields: CommonFields {
        account: "rSenderAddress123".into(),
        transaction_type: TransactionType::Payment,
        ..Default::default()
    },
    amount: Amount::xrp_amount("1000000"), // 1 XRP in drops
    destination: "rDestinationAddress456".into(),
    ..Default::default()
}
.with_fee("12".into())
.with_sequence(100)
.with_destination_tag(12345)
.with_memo(Memo {
    memo_data: Some("payment memo".into()),
    memo_format: None,
    memo_type: Some("text".into()),
});

// Create a cross-currency payment with path finding
let cross_currency_payment = Payment {
    common_fields: CommonFields {
        account: "rSenderAddress123".into(),
        transaction_type: TransactionType::Payment,
        ..Default::default()
    },
    amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
        "USD".into(),
        "rUSDIssuer789".into(),
        "100".into(),
    )),
    destination: "rDestinationAddress456".into(),
    ..Default::default()
}
.with_send_max(Amount::xrp_amount("110000000")) // Max 110 XRP
.with_flag(PaymentFlag::TfPartialPayment)
.with_fee("12".into())
.with_sequence(101);

// Set up an account with deposit authorization
let account_setup = AccountSet {
    common_fields: CommonFields {
        account: "rAccountToSetup123".into(),
        transaction_type: TransactionType::AccountSet,
        ..Default::default()
    },
    ..Default::default()
}
.with_set_flag(AccountSetFlag::AsfDepositAuth)
.with_transfer_rate(1020000000) // 2% transfer fee
.with_fee("12".into())
.with_sequence(50);

// Delete an account
let account_deletion = AccountDelete {
    common_fields: CommonFields {
        account: "rAccountToDelete456".into(),
        transaction_type: TransactionType::AccountDelete,
        ..Default::default()
    },
    destination: "rDestinationAccount789".into(),
    ..Default::default()
}
.with_destination_tag(98765)
.with_fee("2000000".into()) // 2 XRP minimum fee for account deletion
.with_sequence(200)
.with_memo(Memo {
    memo_data: Some("closing account".into()),
    memo_format: None,
    memo_type: Some("text".into()),
});

// Sign and submit transactions
let wallet = Wallet::from_seed("sEdV19BLfeQeKdEXyYA4NhjPJe6XBfG", None, false)?;
let client = XRPLSyncClient::new("https://s.altnet.rippletest.net:51234")?;

let signed_payment = wallet.sign(&payment.into(), Some(true))?;
let submit_response = client.submit(signed_payment)?;
```

### Working with AMM Transactions

```rust
use xrpl::models::transactions::{AMMCreate, AMMBid, AMMDelete};
use xrpl::models::{Amount, Currency, IssuedCurrencyAmount};
use xrpl::models::currency::XRP;

// Create an AMM pool
let amm_create = AMMCreate {
    common_fields: CommonFields {
        account: "rAMMCreator123".into(),
        transaction_type: TransactionType::AMMCreate,
        ..Default::default()
    },
    amount: Amount::XRPAmount(XRPAmount::from("50000000")), // 50 XRP
    amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
        "USD".into(),
        "rUSDIssuer456".into(),
        "50".into(), // 50 USD
    )),
    trading_fee: 100, // 0.1% trading fee
}
.with_fee("12".into())
.with_sequence(100)
.with_memo(Memo {
    memo_data: Some("creating XRP-USD AMM".into()),
    memo_format: None,
    memo_type: Some("text".into()),
});

// Bid on AMM auction slot
let amm_bid = AMMBid {
    common_fields: CommonFields {
        account: "rBidder789".into(),
        transaction_type: TransactionType::AMMBid,
        ..Default::default()
    },
    asset: Currency::XRP(XRP::new()),
    asset2: Currency::IssuedCurrency(IssuedCurrency::new(
        "USD".into(),
        "rUSDIssuer456".into(),
    )),
    ..Default::default()
}
.with_bid_min(IssuedCurrencyAmount::new(
    "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
    "rLPTokenIssuer".into(),
    "100".into(),
))
.with_bid_max(IssuedCurrencyAmount::new(
    "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
    "rLPTokenIssuer".into(),
    "200".into(),
))
.with_fee("15".into())
.with_sequence(200);

// Delete empty AMM
let amm_delete = AMMDelete {
    common_fields: CommonFields {
        account: "rAMMDeleter111".into(),
        transaction_type: TransactionType::AMMDelete,
        ..Default::default()
    },
    asset: Currency::XRP(XRP::new()),
    asset2: Currency::IssuedCurrency(IssuedCurrency::new(
        "USD".into(),
        "rUSDIssuer456".into(),
    )),
    ..Default::default()
}
.with_fee("12".into())
.with_sequence(300);
```

### Address Conversion

```rust
use xrpl::core::addresscodec::{
    classic_address_to_xaddress,
    xaddress_to_classic_address,
    is_valid_classic_address,
};

// Convert classic address to X-address
let classic_address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
let xaddress = classic_address_to_xaddress(classic_address, None, false)?;
println!("X-Address: {}", xaddress);

// Convert X-address back to classic address
let (address, tag, is_test) = xaddress_to_classic_address(&xaddress)?;
println!("Classic Address: {}, Tag: {:?}, Test Network: {}", address, tag, is_test);

// Validate addresses
let is_valid = is_valid_classic_address(classic_address, None);
println!("Address is valid: {}", is_valid);
```

### Working with NFTs

```rust
use xrpl::models::transactions::{NFTokenMint, NFTokenCreateOffer, NFTokenAcceptOffer};
use xrpl::models::Amount;

// Mint an NFT
let nft_mint = NFTokenMint {
    common_fields: CommonFields {
        account: "rNFTMinter123".into(),
        transaction_type: TransactionType::NFTokenMint,
        ..Default::default()
    },
    nftoken_taxon: 0,
    ..Default::default()
}
.with_fee("12".into())
.with_sequence(100)
.with_memo(Memo {
    memo_data: Some("minting unique NFT".into()),
    memo_format: None,
    memo_type: Some("text".into()),
});

// Create an NFT sell offer
let nft_sell_offer = NFTokenCreateOffer {
    common_fields: CommonFields {
        account: "rNFTSeller456".into(),
        transaction_type: TransactionType::NFTokenCreateOffer,
        ..Default::default()
    },
    nftoken_id: "000B013A95F14B0E44F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
    ..Default::default()
}
.with_amount(Amount::xrp_amount("1000000")) // 1 XRP
.with_fee("12".into())
.with_sequence(200);
```

### Binary Codec Usage

```rust
use xrpl::core::binarycodec::{encode, decode};
use serde_json::json;

// Encode transaction to binary
let tx_json = json!({
    "TransactionType": "Payment",
    "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    "Amount": "1000000",
    "Fee": "12",
    "Sequence": 1
});

let encoded = encode(&tx_json, Some(true))?; // true for signing
println!("Encoded transaction: {}", encoded);

// Decode binary back to JSON
let decoded = decode(&encoded)?;
println!("Decoded transaction: {}", serde_json::to_string_pretty(&decoded)?);
```

### Utility Functions

```rust
use xrpl::utils::{
    xrp_to_drops, drops_to_xrp,
    posix_to_ripple_time, ripple_time_to_posix,
};

// XRP conversion
let xrp_amount = "1.5";
let drops = xrp_to_drops(xrp_amount)?;
println!("1.5 XRP = {} drops", drops);

let xrp_back = drops_to_xrp(&drops)?;
println!("{} drops = {} XRP", drops, xrp_back);

// Time conversion
let posix_time = 1660187459;
let ripple_time = posix_to_ripple_time(posix_time)?;
println!("POSIX {} = Ripple time {}", posix_time, ripple_time);

let posix_back = ripple_time_to_posix(ripple_time)?;
println!("Ripple time {} = POSIX {}", ripple_time, posix_back);
```

### Error Handling

```rust
use xrpl::models::exceptions::XRPLModelException;
use xrpl::core::exceptions::XRPLCoreException;
use xrpl::wallet::exceptions::XRPLWalletException;

// Proper error handling example
match Wallet::from_seed("invalid_seed", None, false) {
    Ok(wallet) => println!("Wallet created: {}", wallet.classic_address),
    Err(XRPLWalletException::InvalidSeed(msg)) => {
        eprintln!("Invalid seed provided: {}", msg);
    },
    Err(e) => eprintln!("Other wallet error: {:?}", e),
}

// Transaction validation
let payment = Payment {
    common_fields: CommonFields {
        account: "rSender123".into(),
        transaction_type: TransactionType::Payment,
        ..Default::default()
    },
    amount: Amount::xrp_amount("1000000"),
    destination: "rReceiver456".into(),
    ..Default::default()
}
.with_fee("12".into())
.with_sequence(100);

match payment.validate() {
    Ok(_) => println!("Transaction is valid"),
    Err(e) => eprintln!("Transaction validation failed: {}", e),
}
```

# Contributing [![contributors_status]][contributors]

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Development Setup

```bash
# Clone the repository
git clone https://github.com/sephynox/xrpl-rust.git
cd xrpl-rust

# Run tests
cargo test

# Run CLI tests
cargo test --features cli,std

# Build with all features
cargo build --all-features
```

# License [![license_status]][license]

This project is licensed under the [ISC License](LICENSE).
