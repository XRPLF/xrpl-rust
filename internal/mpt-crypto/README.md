# mpt-crypto

Safe Rust wrappers around the [mpt-crypto](https://github.com/XRPLF/mpt-crypto)
C library, providing the cryptographic primitives needed to construct
[XLS-0096 Confidential Multi-Purpose Token](https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0096-confidential-mpt)
transactions.

This is the public-facing crate that xrpl-rust uses internally. It curates
the ~12 functions a client library actually needs from the ~88 raw FFI
bindings in `mpt-crypto-sys`, and adds idiomatic Rust types, `Result`-based
error handling, and `Zeroize`-on-drop secret material.

## Architecture

This crate sits in the middle of a four-layer stack:

```
┌─────────────────────────────────────────────────────────────┐
│  xrpl-rust transaction models                               │
│  (ConfidentialMPTConvert / Send / ConvertBack / Clawback)   │
└────────────────────────────┬────────────────────────────────┘
                             │  uses safe types and Result
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  mpt-crypto         ◀──── this crate                        │
│  - typed newtypes (Privkey, Pubkey, Ciphertext, …)          │
│  - Result<T, Error>-based error handling                    │
│  - Zeroize-on-drop for secret material                      │
│  - per-transaction-type ProofParams structs                 │
└────────────────────────────┬────────────────────────────────┘
                             │  unsafe extern "C" calls
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  mpt-crypto-sys     (raw FFI, bindgen-generated)            │
│  - 88 `unsafe extern "C"` functions                         │
│  - C-shaped types: `*const u8`, `*mut u8`, raw structs      │
└────────────────────────────┬────────────────────────────────┘
                             │  dynamic linking via rpath
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  libmpt-crypto.{dylib,so,dll}                               │
│  (cryptography in C, secp256k1 + OpenSSL statically inside) │
└─────────────────────────────────────────────────────────────┘
```

Each layer has one job:

| Layer | Concern |
|---|---|
| Native library | Cryptographic algorithms |
| `mpt-crypto-sys` | Make the C symbols callable from Rust |
| **`mpt-crypto`** | **Make the FFI safe and idiomatic** |
| Transaction models | Map cryptographic primitives onto XRPL transaction shapes |

## Module map

| Module | Purpose | Key items |
|---|---|---|
| [`types`]   | Strongly-typed byte-array newtypes | `Privkey`, `Pubkey`, `Ciphertext`, `Commitment`, `ContextHash`, `AccountId`, `IssuanceId`, `BlindingFactor`, the four `*Proof` types |
| [`error`]   | Error enum + `Result` alias | `Error::NonZeroRc(i32)`, `Error::Invariant(&'static str)` |
| [`keypair`] | ElGamal/secp256k1 keypair generation | `keypair::generate() -> Result<(Privkey, Pubkey)>` |
| [`encrypt`] | EC-ElGamal encrypt / decrypt + RNG | `encrypt::{encrypt, decrypt, random_blinding_factor}` |
| [`commit`]  | Pedersen commitments | `commit::pedersen(amount, blinding) -> Result<Commitment>` |
| [`context`] | Per-transaction context hashes (replay-binding) | `context::{convert, convert_back, send, clawback}` |
| [`prove`]   | The four per-transaction proof generators | `prove::{convert, send, convert_back, clawback}` plus `Participant`, `SendProofParams`, `ConvertBackProofParams` |

`lib.rs` re-exports `Error`, `Result`, and everything in `types`, so most
consumers can `use mpt_crypto::{Privkey, Pubkey, …}` directly without
naming `types`.

## Type discipline

Two flavors of newtype, distinguished by sensitivity:

### Public-information types — built with the `public_bytes!` macro

`Pubkey`, `Ciphertext`, `Commitment`, `ContextHash`, `AccountId`,
`IssuanceId`, and the four `*Proof` types. These are:

- `#[derive(Clone, Copy, PartialEq, Eq, Hash)]` — cheap to pass around.
- `Debug` — abbreviated as `Pubkey(0xab12cd34…ef56gh78)` (first 4 + last 4
  bytes), to keep log output readable, **not** for security.

### Secret types — built with the `secret_bytes!` macro

`Privkey` and `BlindingFactor`. These are:

- **Not `Copy`** — moves are explicit; you can't accidentally clone-by-copy.
- `#[derive(Clone, Zeroize, ZeroizeOnDrop)]` — bytes are wiped from RAM
  when the value goes out of scope (volatile writes, defeating the
  compiler's dead-store elimination).
- `Debug` — redacted as `Privkey(<redacted>)`. Bytes never leak through
  formatting.

The `privkey_debug_does_not_leak_bytes` integration test enforces the
redaction contract.

## Error model

Single error type with two variants:

```rust
pub enum Error {
    /// FFI returned a non-zero status code (the C contract is "0 on success,
    /// -1 on failure"; we surface the raw value rather than coercing it).
    NonZeroRc(i32),

    /// A post-condition the FFI promised wasn't met (e.g. the Send proof
    /// utility wrote a different number of bytes than the spec advertises).
    Invariant(&'static str),
}
```

The split is intentional: `NonZeroRc` means *"the C side told us it failed"*;
`Invariant` means *"the C side said it succeeded but the result violates an
assumption we baked in."* They imply different debugging paths.

## Quick example

```rust
use mpt_crypto::{keypair, encrypt, commit, context, prove, AccountId, IssuanceId};

let (sender_sk, sender_pk) = keypair::generate()?;
let r = encrypt::random_blinding_factor()?;

// Encrypt the same amount under two different keys with the same r —
// matches the XLS-0096 §5.4 shared-randomness pattern.
let amount = 1_000;
let ct_under_sender = encrypt::encrypt(amount, &sender_pk, &r)?;

// Pedersen commit + per-tx context hash + Convert-style Schnorr PoK.
let commitment = commit::pedersen(amount, &r)?;
let acct       = AccountId::new([0u8; 20]);   // real values come from xrpl-rust
let iss        = IssuanceId::new([0u8; 24]);
let ctx        = context::convert(&acct, &iss, /* sequence */ 1)?;
let proof      = prove::convert(&sender_sk, &sender_pk, &ctx)?;
```

End-to-end working examples for all four transaction types live in
`tests/integration.rs`.

## Running tests

This crate has 14 integration tests (no inline unit tests; everything goes
through the public API).

```bash
# All tests
cargo test -p mpt-crypto

# Just the convert_back tests (positive + overdraft rejection)
cargo test -p mpt-crypto convert_back

# A single test by full name
cargo test -p mpt-crypto convert_proof_rejects_wrong_pubkey

# Show stdout/println output (otherwise hidden by libtest)
cargo test -p mpt-crypto -- --nocapture

# Verbose: see linker invocations from build.rs
cargo test -p mpt-crypto -v
```

### What the tests verify

| Group | What it covers |
|---|---|
| `keypair_*`, `encrypt_*` | ElGamal encrypt/decrypt round-trip |
| `pedersen_*` | Commitment determinism and binding |
| `*_debug_*` | Type-level redaction of secrets vs. abbreviation of public values |
| `context_hashes_*`, `send_context_hash_*` | Domain separation across transaction types and binding to destination |
| `convert_proof_verifies` | Schnorr PoK round-trip (prove → C verifier accepts) |
| `convert_proof_rejects_wrong_pubkey` | Convert proof binds to pk; can't be retargeted |
| `convert_back_proof_verifies` | 816 B composite proof round-trip |
| **`convert_back_proof_rejects_withdrawal_exceeding_balance`** | **No overdraft proof can be generated AND accepted** |
| `clawback_proof_verifies` | 64 B compact sigma round-trip |
| `send_proof_verifies_three_participants` | 946 B composite proof round-trip (no auditor) |

The proof tests use the C-side `mpt_verify_*` functions as oracles — if the
verifier accepts the proof we generated, the FFI plumbing and the
cryptographic algorithms are coherent.

### What's required to run tests

- The `mpt-crypto-sys` crate must be buildable (see its
  [README](../mpt-crypto-sys/README.md) for first-time setup; on a fresh
  clone, `build.rs` will download `libmpt-crypto.{so,dylib,dll}` from the
  pinned upstream release the first time you build).
- A network connection on first build, unless you've populated
  `internal/mpt-crypto-sys/vendor/lib/<target>/` locally or set
  `MPT_CRYPTO_LIB_DIR`.
- No `libclang` / `bindgen-cli` needed (the sys crate ships pre-generated
  bindings).

If something fails at runtime with "Library not loaded" / "error while
loading shared libraries", see the **Verifying the build linked correctly**
section of `internal/mpt-crypto-sys/README.md`.

## Why `publish = false`

This crate is a private workspace member of `xrpl-rust`. It depends on
`mpt-crypto-sys` via path, which is also unpublished. The intent is for
xrpl-rust transaction models to consume this crate internally, not for it
to be a standalone library on crates.io.

If you need to depend on confidential-MPT crypto from outside xrpl-rust,
the recommended path is a git dependency on the parent xrpl-rust workspace.

## See also

- [`mpt-crypto-sys` README](../mpt-crypto-sys/README.md) — the FFI layer
  this crate wraps; covers native-library distribution, build.rs internals,
  and the bindgen regeneration procedure.
- [XLS-0096 spec](https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0096-confidential-mpt) —
  the cryptographic protocol this crate implements wrappers for.
- [mpt-crypto upstream](https://github.com/XRPLF/mpt-crypto) — the C
  library; its `paper/cmpt-compact-sigma.pdf` is the formal spec for the
  compact AND-composed sigma proof used by Send/ConvertBack/Clawback.
