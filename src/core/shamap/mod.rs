//! ShaMap: a radix-16 Merkle trie used in the XRP Ledger.
//!
//! This module implements an optimized ShaMap using HAMT bitmap indexing,
//! hash caching with dirty-flag propagation, and single-buffer hashing.
//!
//! - [`ShaMap`] -- the top-level hash tree for building transaction and account
//!   state trees. Supports O(1) cached `hash()`, incremental add/remove, and
//!   Merkle inclusion proofs.
//! - [`hash_prefix`] -- domain-separation constants for all XRPL hash contexts
//! - [`sha512half`] -- the SHA-512/256 hash primitive
//! - [`ledger`] -- ledger header hashing and tree construction helpers
//! - [`ShaMapProof`] and [`verify_proof`] -- SPV-style inclusion proofs

pub mod hash_prefix;
pub mod ledger;
pub mod sha512half;
pub mod tree;

pub use self::ledger::{
    account_state_hash, ledger_hash, transaction_tree_hash, AccountStateItem, LedgerHeader,
    TransactionItem,
};
pub use self::sha512half::{sha512half as sha512half_fn, Sha512Half};
pub use self::tree::{
    verify_proof, Hash256, ProofLevel, ShaMap, ShaMapIndex, ShaMapInner, ShaMapLeaf, ShaMapNode,
    ShaMapProof, ZERO_256,
};
