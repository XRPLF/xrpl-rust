# Copilot Instructions for xrpl-rust

## Project Overview

`xrpl-rust` is a 100% Rust, `no_std`-compatible library for interacting with the XRP Ledger.
The crate name is `xrpl-rust` but the lib is imported as `xrpl` (e.g., `use xrpl::models::...`).
Check `Cargo.toml` for the current version and edition. Toolchain: stable (see CI for pinned version).

## Performance

Always use all available CPU cores for cargo commands. Detect at runtime:

```bash
JOBS=$(nproc)
cargo build --release -j $JOBS
cargo test --release -j $JOBS
cargo clippy --all-features -j $JOBS -- -D warnings
```

Do not hardcode core counts. Use `nproc` (Linux) or `sysctl -n hw.ncpu` (macOS) to maximize parallelism on any machine.

## Architecture

```
src/
  core/           — Binary codec, keypairs, address codec
  models/
    transactions/ — One file per transaction type (e.g., account_delete.rs)
    ledger/
      objects/    — One file per ledger entry type (e.g., amm.rs)
    requests/     — RPC request models
    amount.rs     — XRPAmount, Amount, Currency types
  utils/          — Conversion helpers
  asynch/         — Async client (WebSocket, JSON-RPC)
tests/
  transactions/   — Integration tests per transaction type
xrpl-rust-macros/ — Proc macros (ValidateCurrencies, serde_with_tag!)
```

## no_std Compatibility (CRITICAL)

This crate is `no_std` with `alloc`. You must:

- Use `alloc::borrow::Cow<'a, str>` instead of `String` for all string fields
- Use `alloc::vec::Vec` instead of `std::vec::Vec` — always import explicitly
- Use `alloc::string::ToString` when needed
- Never use `std::` imports in model code
- Every file that uses `Vec` must have `use alloc::vec::Vec;`

## Transaction Type Pattern

Every transaction follows this exact pattern. Use `src/models/transactions/account_delete.rs` as the canonical template:

```rust
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};
use super::{CommonFields, CommonTransactionBuilder};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone,
         xrpl_rust_macros::ValidateCurrencies)]
#[serde(rename_all = "PascalCase")]
pub struct MyTransaction<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // Transaction-specific fields here
}

impl<'a> Model for MyTransaction<'a> { ... }
impl<'a> Transaction<'a, NoFlags> for MyTransaction<'a> { ... }
impl<'a> CommonTransactionBuilder<'a, NoFlags> for MyTransaction<'a> { ... }
```

Key conventions:
- `#[serde(rename_all = "PascalCase")]` on all structs
- `#[serde(rename = "FieldName")]` only when PascalCase conversion differs (e.g., `URI`, `DomainID`, `NFTokenID`, `LPToken`)
- Use `NoFlags` for transactions without flags; use a flags enum with `IntoEnumIterator` when flags exist
- Implement `new()` constructor and builder methods (`with_*`)
- Derive `ValidateCurrencies` if the struct contains `Amount` or `Currency` fields

## Ledger Entry Pattern

Use `src/models/ledger/objects/amm.rs` as the template. Same pattern as transactions but with ledger-specific `CommonFields`.

## Nested Tagged Types (STArray of STObjects)

For XRPL array-of-object fields (like `PriceDataSeries`, `AcceptedCredentials`), use the `serde_with_tag!` macro:

```rust
serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Clone, Default)]
    pub struct PriceData {
        pub base_asset: Option<String>,
        pub quote_asset: Option<String>,
    }
}
```

Define these in `src/models/transactions/mod.rs`. See `AuthAccount`, `VoteEntry` in `src/models/ledger/objects/amm.rs` for examples.

## Registration Checklist

When adding a new transaction or ledger entry type, you must register it in:

1. **`src/models/transactions/mod.rs`**: Add `pub mod my_transaction;`, add variant to `TransactionType` enum, add to `Transaction` enum with `serde_with_tag!`, add re-export
2. **`src/models/ledger/objects/mod.rs`**: Add `pub mod my_ledger_entry;`, add variant to `LedgerEntryType` enum (with correct hex code from definitions.json), add to `LedgerEntry` enum
3. **`tests/transactions/mod.rs`**: Add `mod my_transaction;` for integration tests

Missing any registration will cause serialization/deserialization failures.

## XLS Spec Compliance

When implementing a new XLS feature (e.g., XLS-47, XLS-65, XLS-70, XLS-80):

1. **Find and read the official spec** at `https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-NNNN-<name>/`
2. **Cross-reference with `definitions.json`** in this repo (`src/core/binarycodec/definitions.json`) for field codes, type codes, and serialization order
3. **Verify every field** in the spec is present in the implementation with the correct type:
   - `AccountID` → `Cow<'a, str>`
   - `Amount` → `Amount<'a>` (use `ValidateCurrencies` derive)
   - `Blob` → `Cow<'a, str>` (hex-encoded)
   - `Hash256` → `Cow<'a, str>`
   - `UInt8` → `u8`, `UInt16` → `u16`, `UInt32` → `u32`
   - `UInt64` → `Cow<'a, str>` (hex string — JavaScript can't handle u64)
   - `STArray` of `STObject` → `Vec<T>` where T uses `serde_with_tag!`
   - `Issue`/`Currency` → `Currency<'a>`
   - `Number` → `Cow<'a, str>` (string representation)
4. **Check field optionality**: Fields that are optional in the spec should be `Option<T>`. Required fields may still be `Option<T>` in the SDK (server validates), but prefer non-optional for fields that are always required by the protocol.

## Reference Implementations

When implementing a new feature, always cross-reference the test suites in the Python and JavaScript SDKs for test coverage ideas:

- **xrpl-py**: `https://github.com/XRPLF/xrpl-py` — check `tests/unit/models/transactions/` for the equivalent transaction type
- **xrpl.js**: `https://github.com/XRPLF/xrpl.js/tree/main` — check `packages/xrpl/test/models/` for the equivalent transaction type

Adapt their test cases to Rust. These libraries often have validation tests, edge cases, and error-condition tests that should also exist in xrpl-rust.

## Required Tests

Every new type must include both positive (happy path) and negative (error path) tests. Aim for 95%+ line coverage on all new code.

### In-file unit tests (`#[cfg(test)] mod tests`)

**Positive tests (happy path):**
- **Serde roundtrip**: Serialize to JSON, deserialize back, assert equality
- **JSON format**: Verify exact JSON field names match PascalCase XRPL convention
- **`get_transaction_type()`**: Assert it returns the correct `TransactionType` variant (prevents mutation testing escapes)
- **Builder pattern**: Test `with_*` methods preserve all fields
- **`new()` constructor**: Test all parameters are correctly assigned
- **Default construction**: Verify defaults are sensible
- **Optional fields**: Test with and without optional fields set

**Negative tests (error paths):**
- **Validation failures**: Test `get_errors()` returns specific errors for each invalid state (missing required fields, out-of-range values, invalid combinations)
- **Malformed JSON**: Test deserialization of invalid/incomplete JSON returns `Err`, not panic
- **Boundary values**: Test with empty strings, zero-length vectors, `u32::MAX`, fields at their protocol limits
- **Invalid field combinations**: Test mutually exclusive fields, fields that require other fields to be present
- **Wrong transaction type**: Verify deserialization rejects JSON with a mismatched `TransactionType`

### Property-based tests (proptest)
- **Dependency**: `proptest` must be in `[dev-dependencies]` in `Cargo.toml`. If it is not already present, add `proptest = "1"` before writing proptests.
- Use the `proptest` crate to generate arbitrary valid and invalid inputs
- Test that serialization roundtrips hold for all generated inputs: `deserialize(serialize(x)) == x`
- Test that validation never panics regardless of input (returns `Ok` or `Err`, never crashes)
- Test boundary conditions: empty strings, maximum-length fields, u32::MAX values, empty vectors
- Example pattern:
  ```rust
  use proptest::prelude::*;

  proptest! {
      #[test]
      fn roundtrip_never_panics(
          account in "[a-zA-Z0-9]{25,35}",
          fee in "[0-9]{1,10}",
          seq in any::<u32>(),
      ) {
          let txn = MyTransaction { /* construct from generated values */ };
          let json = serde_json::to_string(&txn).unwrap();
          let back: MyTransaction = serde_json::from_str(&json).unwrap();
          prop_assert_eq!(txn, back);
      }

      #[test]
      fn validation_never_panics(
          account in ".*",
          field in ".*",
          val in any::<u32>(),
      ) {
          let txn = MyTransaction { /* construct with arbitrary values */ };
          // Must return Ok or Err — never panic
          let _ = txn.get_errors();
      }
  }
  ```

### Integration tests (`tests/transactions/<name>.rs`)
- Serde roundtrip with realistic field values
- Binary codec roundtrip if `codec-fixtures.json` has test vectors for the type

## CI Pipeline (Must All Pass)

All cargo commands should use `-j $(nproc)` to maximize core utilization.

### 1. Build & Lint (`build_and_lint.yml`)
```bash
cargo fmt --all -- --check
cargo clippy --all-features -j $(nproc) -- -D warnings

# Default and no_std feature matrix — all must compile cleanly
FEATURE_SETS=(
  ""
  "--no-default-features"
  "--no-default-features --features embassy-rt,core,wallet,models,helpers,websocket,json-rpc"
  "--no-default-features --features core"
  "--no-default-features --features wallet"
  "--no-default-features --features models"
  "--no-default-features --features websocket,json-rpc,helpers,tokio-rt"
  "--no-default-features --features websocket"
  "--no-default-features --features json-rpc"
)
for features in "${FEATURE_SETS[@]}"; do
  cargo build --release -j $(nproc) $features
done
```

New code must compile under every feature combination above. The `--no-default-features` builds verify `no_std` compatibility — missing `alloc::` imports will fail here even if default-feature builds succeed.

### 2. Unit Tests (`unit_test.yml`)
```bash
# Default features
cargo test --release -j $(nproc)
# no_std features
cargo test --release -j $(nproc) --no-default-features --features embassy-rt,core,utils,wallet,models,helpers,websocket,json-rpc
```

### 3. Coverage (`unit_test.yml`)
```bash
cargo llvm-cov --summary-only \
  --fail-under-lines 95 \
  --fail-under-regions 95 \
  --fail-under-functions 95
```

**All new code must achieve 95%+ coverage across lines, regions, and functions.** Run `cargo llvm-cov --summary-only` locally before submitting. Use `cargo llvm-cov --open` to inspect uncovered lines and add tests to close gaps.

### 4. Integration Tests (`integration_test.yml`)
```bash
# Start rippled standalone node — the .ci-config volume mount is required
# so rippled binds to 0.0.0.0 (reachable from the host)
docker run --detach --rm \
  -p 5005:5005 -p 6006:6006 \
  --volume "$PWD/.ci-config/:/etc/opt/ripple/" \
  --name rippled-service \
  --health-cmd="rippled server_info || exit 1" \
  --health-interval=5s --health-retries=10 --health-timeout=2s \
  --entrypoint bash rippleci/rippled:develop \
  -c "mkdir -p /var/lib/rippled/db/ && rippled -a"

# Wait for healthy
until docker inspect --format='{{.State.Health.Status}}' rippled-service | grep -q healthy; do
  sleep 2
done

# Run integration tests (serialized — not parallel)
cargo test --release -j $(nproc) --features std,json-rpc,helpers,integration --test integration_test -- --test-threads=1

# Cleanup
docker stop rippled-service
```

### 5. Quality Check (`quality_test.yml`)
```bash
cargo audit
```

## Zero Warnings Policy

All code must compile and lint with zero warnings:
- `cargo clippy --all-features -j $(nproc) -- -D warnings`
- `cargo build --release -j $(nproc)` with no warnings
- `cargo fmt --all -- --check` must pass with no diff
- Never suppress warnings with `#[allow(...)]` unless there is a documented, unavoidable reason

## Code Quality Standards

- **DRY**: Check for existing functions, traits, or types before writing new ones
- **Idiomatic Rust**: Use `?` over `unwrap()`, iterators over manual loops, `Cow` for zero-copy
- **Security**: No `unwrap()`/`expect()` on untrusted input. Validate at system boundaries.
- **Documentation**: Add doc comments with `///` on all public types and fields. Include links to XRPL docs (e.g., `/// See AccountDelete: <https://xrpl.org/docs/references/protocol/transactions/types/accountdelete>`)
- **Naming**: Match XRPL field names exactly in serde (PascalCase). Rust field names use snake_case.
- **Error handling**: Use the crate's error types (`XRPLModelException`, `XRPLModelResult`). Return meaningful error messages that identify the invalid field and why it failed.

## PR Requirements

- Every PR must pass all 4 CI workflows
- Include unit tests (positive and negative paths) alongside implementation
- Include integration tests in `tests/transactions/`
- Include property-based tests for new types
- Verify binary codec roundtrip if codec-fixtures.json has test vectors
- Run `cargo llvm-cov --summary-only` and confirm new code has 95%+ line, region, and function coverage
- Get a full code review from a maintainer before merging
