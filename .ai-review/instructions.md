# Reviewing xrpl-rust

xrpl-rust is the canonical **Rust SDK for the XRP Ledger** (crate `xrpl` + the
`xrpl-rust-macros` proc-macro crate), **no_std-first**. It is a **financial primitive**: amount,
serialization, and signing bugs corrupt transactions or break consensus compatibility, and
surface only on-ledger. **rippled is the source of truth** — verify wire shapes against it,
not idiomatic serde design.

## Amounts & numbers
- Amount values are `Cow<'a, str>` strings, never numeric fields. XRPAmount's `get_errors`/
  `TryInto<u32>` parse drops as **u32 — a known-bad precedent** rejecting amounts > ~4,294 XRP;
  flag new drops code copying it; use u64/BigDecimal.
- `From<f64>` is unit-asymmetric: `Amount::from(f64)` = **XRP** (×1e6); `XRPAmount::from(f64)` =
  verbatim **drops** — wrong constructor = silent 1e6 error.
- Both decimal crates are deliberate: `rust_decimal` for XRP/drops (utils only), **BigDecimal
  for IOU** (IOU exponents −96..80 exceed it). Don't consolidate; no rust_decimal/f64 in new IOU paths.
- `Amount` deserialization is hand-written exact-key-set dispatch (MPT = exactly `{mpt_issuance_id,
  value}` first, IOU = exactly 3 keys, XRP string last); `deny_unknown_fields` on the amount
  structs is **load-bearing** too. Don't flag it (xrpl.js parity); DO flag relaxing it or field
  additions that skip the key-count guards in `amount/mod.rs`.

## Binary codec
- Round-trip plus **byte-exact xrpl.js reference vectors are the spec**. A new SerializedType needs
  **three registration arms** in `types/mod.rs` (XRPLTypes enum, `from_value` match,
  `From<XRPLTypes>`) — unregistered types fail only at runtime.
- `BinaryParser::read(n)` **panics** on truncated input — validate wire-derived lengths first.
  Every field decode must **consume exactly the field's bytes or Err**; `Ok` without
  advancing corrupts every later field.
- Canonical ordinal sort and signing-field filtering happen ONLY in `STObject::try_from_value`
  — flag encode paths calling `write_field_and_value` directly, bypassing `serialize_json`.
- UInt64 strings are **hex by default**; only `BASE10_UINT64_FIELDS` (MaximumAmount,
  OutstandingAmount, MPTAmount, LockedAmount) are base-10. Don't flag the asymmetry (xrpl.js
  parity); DO flag a new decimal-string u64 field not added there — it misparses as hex.
- IOU Amount encoding **rounds underflow to canonical zero** (0x8000…) and errors on overflow
  — spec, not precision loss. Mantissa/exponent math has a fixed-bug history (unsigned_abs sign
  loss, IOU sign inversion): changes need exponent-boundary and zero-crossing
  tests; range-check wire-read integers where overflow is possible (panics debug builds).

## definitions.json
- **No generator**: `definitions.json` is a manual copy synced from xrpl.js/rippled, embedded via
  `include_str!`. Hand-edits are normal here; drift from upstream is the finding.
- A FIELDS entry whose `"type"` is missing from TYPES **panics the whole codec** at first use; a new
  TYPES name needs an `XRPLTypes::from_value` arm; model support for new tx/ledger-entry types
  needs variants in the **hardcoded** `TransactionType`/`LedgerEntryType` enums.

## Transaction models
- New tx type = model file + `pub mod` + `TransactionType` variant, whose **name IS the wire
  string** (plain serde, no renames): exact rippled casing (NFTokenMint, DIDSet) or it
  serializes wrong with no compile error.
- Struct shape: `#[skip_serializing_none]` + PascalCase rename + flattened `CommonFields<'a,
  FlagEnum>`. Acronym fields need explicit renames (`AccountTxnID`, `URI`) — a missing rename
  round-trips against itself but **rippled rejects it**.
- Flags are `#[repr(u32)]` serde_repr enums in `FlagCollection` (wire = one u32 bitmask); `EnumIter`
  is load-bearing for decoding. Flagless txs use `NoFlags`.
- Models deriving `ValidateCurrencies` must chain `self.validate_currencies()` in
  `get_errors()` — the derive **generates the method but nothing calls it**. It matches type
  names **textually** (six names, bare `Option<T>` only): aliases, `Vec<Amount>`, or new types
  are silently skipped — flag amount/currency fields typed outside the six.

## Cryptography & signing
- The algorithm is inferred **only from encoding prefixes** (seed `sEd…` → ed25519, `s…` →
  secp256k1; key prefix `ED` vs zero-padding); `generate_seed` defaults to ED25519. Prefix/format
  changes silently derive a **different address from the same entropy**.
- Prehash asymmetry is spec: secp256k1 signs `sha512_first_half(msg)`, ed25519 signs the raw
  message. RFC-6979 and low-S output come from the secp256k1 crate defaults —
  no canonicalization code is "missing".
- Key material comes from **OsRng/thread_rng only** (a seedable PRNG was a fixed vuln). Error variants
  must never capture secret bytes (Debug-formatting dumps them); long-lived key holders follow
  Wallet: zeroize-on-Drop + redacting Debug.
- Multisigning: 4-byte prefixes (STX/SMT/CLM/BCH) domain-separate payloads + the signer's
  decoded-AccountId suffix; the combined tx gets `SigningPubKey: ""` (present, empty);
  `Signers` sort by **decoded AccountId bytes**, not base58 strings.
- `sign(tx, wallet, true)` **replaces** the Signers array (despite its doc comment);
  `multisign()` reads only the first signer per entry: one tx clone per signer. Flag repeated
  `sign(.., true)` on one object, and autofill-then-multisign without `signers_count` or an
  explicit fee (underpays the fee).
- `autofill` fills only-when-None; **mainnet gets no NetworkID** (id > 1024 gate) —
  intentional. `submit_and_wait` unconditionally unwraps `last_ledger_sequence` — flag
  non-autofill callers without it preset.

## Clients & results (tolerant)
- Result models must deserialize **real rippled responses**: no `deny_unknown_fields` (and broken in
  `#[serde(flatten)]` chains), `Option` for non-guaranteed fields, and
  `serde_json::Value` for unstable sections.
- `XRPLResult` is untagged and **order-dependent** (`Other` stays before Subscribe/Ping); the
  `TryFrom` re-parse-from-`Other` fallbacks are deliberate, not dead code; new variants must
  respect ordering and add a fallback when mis-routable.
- Clients return `Ok` on rippled app errors; `try_into::<XRPLResult>()` does the status check —
  don't flag "missing error handling" at `request_impl`.

## Sync/async parity
- `src/{account,ledger,transaction,wallet}` sync helpers are pure `embassy_futures::block_on`
  facades over `async_`-aliased imports from `src/asynch/` — signatures and lifetimes
  hand-duplicated, **no parity check**. Flag one-sided signature edits and logic in a wrapper —
  behavior lives async-side only. Embassy's `block_on` is the no_std-correct choice, not tokio's.

## no_std & features
- Library code imports `String`/`Vec`/`format` from `alloc::` and the rest from `core::`, never
  `std::` (std only under `cfg(feature = "std")` or in tests). `extern crate std as alloc`
  in std builds is intentional — don't flag it.
- `crate::_serde::HashMap` (hashbrown) for hash maps in model/serde code; IndexMap where
  insertion order matters.
  Errors derive `thiserror_no_std::Error`, never `thiserror`.
- New std-defaulting non-optional deps: `default-features = false` (+ `alloc`), std via the
  `[features] std` list; std-only deps stay `optional = true`.

## Test conventions (do NOT flag)
- `snoPBrXtMeMyMHUVTgbuqAfg1SUTb` / `rHb9…` are the standalone-node genesis credentials,
  intentionally committed. Manual `ledger_accept()`, the close-time poll, `--test-threads=1`,
  and the blockchain lock are required by the shared standalone `xrpld` node.
- Opaque hex vectors annotated as xrpl.js ports ARE the spec — never simplify or regenerate.
  Shared fixtures live in `tests/common/constants.rs`; one-off literals with an inline
  decode comment are accepted style.

## Contributor conventions
- Breaking serde/public-API changes need a CHANGELOG entry (prefer `**Breaking:**`).
