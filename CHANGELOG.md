# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [[Incomplete]]

- Performance Benchmarks
- Utility functions

## [[Unreleased]]

### Added

- New `xrpl::signing` module containing the pure-crypto signing helpers (`sign`, `multisign`, `prepare_transaction`) extracted from `asynch::transaction` and `transaction`. Available with just `core + models + wallet` features (no `helpers`/runtime/client dependency). The legacy paths `asynch::transaction::sign` and `transaction::multisign` are preserved as re-exports for backward compatibility.
- Expanded unit-test coverage and raised CI thresholds: lines `73 → 83`, regions `75 → 85`, functions `67 → 73`.
- Codecov integration with per-PR project (≥83%) and patch (≥80% on new/modified lines) gates.
- Integration-test coverage gate: a CI workflow runs all five integration test binaries under `cargo-llvm-cov`, uploads to codecov under an `integration` flag, and gates the project at ≥65%.

### Changed

- Unit-test and integration-test coverage are now scoped via Cargo feature flags rather than path regex. The unit-test workflow builds with `--no-default-features --features std,core,utils,wallet,models`, so integration-territory code (CLI, async clients, sync wrappers, faucet) simply isn't compiled and doesn't appear in the unit coverage report.
- Network-dependent inline tests in `src/asynch/transaction/` and `src/asynch/wallet/` (`test_autofill_txn`, `test_autofill_and_sign`, `test_submit_and_wait`, `test_generate_faucet_wallet`) are now gated behind `feature = "integration"` so `cargo test --release` is hermetic by default.
- Codecov **patch** coverage is now gated per flag (separate `unit` and `integration` sections) rather than a single combined gate.

### Fixed

- `RipplePathFind::destination_amount` changed from `Currency<'a>` to `Amount<'a>` to match the XRPL wire format.
- `NoRippleCheckRole` no longer serializes with the `#[serde(tag = "role")]` discriminator; now emits a plain `snake_case` string matching the XRPL wire format.
- `is_success()` now reports success correctly for responses deserialized into typed `XRPLResult` variants (e.g. `ServerInfo`); it consults the preserved raw result JSON instead of the re-serialized typed value.
- `get_latest_open_ledger_sequence` now uses the `ledger_current` request; it previously sent `ledger { ledger_index: "open" }`, which rippled rejects with `invalidParams`.

## [[v1.1.0]]

- `DepositPreauth` ledger object: `authorize` field changed from `Cow<'a, str>` to `Option<Cow<'a, str>>` to support XLS-70 credential-based preauthorization. The `new()` constructor is unchanged (still accepts non-optional `authorize`), but direct struct construction must wrap the value in `Some(...)`.
- `credential_ids` field on `AccountDelete`, `Payment`, `EscrowFinish`, `PaymentChannelClaim`, and `credentials` on `DepositAuthorized` request changed from `Option<Cow<'a, [Cow<'a, str>]>>` to `Option<Vec<Cow<'a, str>>>` for reliable serde round-trip.

### Added

- Implemented full deserialization from hex binary back to JSON, update `definitions.json` to `xrpl.js` latest, added all codec test fixtures from xrpl.js and implemented tests for all of them.
- Added integration tests for all transaction types, refactored to separate files.
- Added initial XLS-70 Credentials model support (`CredentialCreate`, `CredentialAccept`, `CredentialDelete`, `Credential` ledger object, and credential-based `DepositPreauth` fields).

### Fixed

- Fixed serialization issues for `PathSet`, `Issue`, and `STArray` types.

## [[v1.0.0]]

- Initial production release
- command line interface
- automated market maker
- utility functions
- sidechain support

## [[v.0.6.0]]

- Added CLI interface
- missing network_id member added to server info response
- server_state_duration_us in server info type changed to str

## [[v0.5.0]]

- add missing NFT request models
- add `parse_nftoken_id` and `get_nftoken_id` utility functions
- complete existing result models and add NFT result models
- add transaction `Metadata` models
- fix serialization issue where null values were tried to be serialized
- fix multisigning bug, because `signing_pub_key` is not set for multisigning but it is required, so it's just an empty string
- add transaction response models
- add integration tests with XRPL test net.

## [[v0.4.0]]

- add amm support
  - Transaction models
  - Transaction signing
  - Request models
- add sidechain support
  - Transaction models
  - Transaction signing
- improve errorhandling utilizing thiserror
- simplifying feature flags

## [[v0.3.0]]

- Examples
  - Wallet from seed
  - New wallet generation
  - Client requests
- make `new` methods of models public
- add `AsyncWebSocketClient` and `WebSocketClient`
- add `AsyncJsonRpcClient` and `JsonRpcClient`
- update dependencies
- add devcontainer
- add transaction helpers and signing
- add account helpers
- add ledger helpers
- add wallet helpers

---

## [[v0.2.0-beta]]

### Added

- Request models
- Transaction models
- Ledger models
- Utilize `anyhow` and `thiserror` for models
- Utilities regarding `serde` crate
- Utilities regarding `anyhow` crate

### Changed

- Use `serde_with` to reduce repetitive serialization skip attribute tags
- Use `strum_macros::Display` instead of manual `core::fmt::Display`
- Use `strum_macros::Display` for `CryptoAlgorithm` enum
- Separated `Currency` to `Currency` (`IssuedCurrency`, `XRP`) and `Amount` (`IssuedCurrencyAmount`, `XRPAmount`)
- Make `Wallet` fields public
- Updated crates:
  - secp256k1
  - crypto-bigint
  - serde_with
  - criterion

### Fixed

- Broken documentation link
- Flatten hex exceptions missed from previous pass

---

## [v0.1.1] - 2021-10-28

Initial core release.

### Added

- All Core functionality working with unit tests
