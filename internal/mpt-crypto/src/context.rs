//! Per-transaction-type context hashes.
//!
//! Each XLS-0096 confidential transaction binds its ZK proof to a 32-byte
//! transcript that includes the transaction type and the relevant ledger
//! identifiers. This prevents replay across contexts and produces clear
//! "wrong context = invalid proof" failures.
//!
//! The four functions here mirror the four context-hash variants the spec
//! defines — they differ in which ledger identifiers are folded into the
//! transcript:
//!
//! | Function | Inputs |
//! |---|---|
//! | [`convert`]      | `account, issuance, sequence` |
//! | [`convert_back`] | `account, issuance, sequence, version` |
//! | [`send`]         | `sender, issuance, sequence, destination, version` |
//! | [`clawback`]     | `issuer, issuance, sequence, holder` |

use crate::{Error, Result, types::{AccountId, ContextHash, IssuanceId}};
use mpt_crypto_sys as sys;

#[inline]
fn into_sys_account(id: &AccountId) -> sys::account_id {
    sys::account_id { bytes: *id.as_bytes() }
}

#[inline]
fn into_sys_issuance(id: &IssuanceId) -> sys::mpt_issuance_id {
    sys::mpt_issuance_id { bytes: *id.as_bytes() }
}

/// Context hash for `ConfidentialMPTConvert`.
///
/// Transcript includes the converting account, the MPTokenIssuance ID, and
/// the transaction sequence number.
pub fn convert(account: &AccountId, issuance: &IssuanceId, sequence: u32) -> Result<ContextHash> {
    let mut out = [0u8; 32];
    // SAFETY: account_id / mpt_issuance_id structs are passed by value and
    //         contain only fixed-size byte arrays; out buffer is 32 bytes.
    let rc = unsafe {
        sys::mpt_get_convert_context_hash(
            into_sys_account(account),
            into_sys_issuance(issuance),
            sequence,
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ContextHash::new(out))
}

/// Context hash for `ConfidentialMPTConvertBack`. Adds `version` to bind the
/// proof to a specific spending-balance state.
pub fn convert_back(
    account: &AccountId,
    issuance: &IssuanceId,
    sequence: u32,
    version: u32,
) -> Result<ContextHash> {
    let mut out = [0u8; 32];
    // SAFETY: see `convert`.
    let rc = unsafe {
        sys::mpt_get_convert_back_context_hash(
            into_sys_account(account),
            into_sys_issuance(issuance),
            sequence,
            version,
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ContextHash::new(out))
}

/// Context hash for `ConfidentialMPTSend`. Binds the proof to a specific
/// `destination` so a sender can't re-target a generated proof.
pub fn send(
    sender: &AccountId,
    issuance: &IssuanceId,
    sequence: u32,
    destination: &AccountId,
    version: u32,
) -> Result<ContextHash> {
    let mut out = [0u8; 32];
    // SAFETY: see `convert`.
    let rc = unsafe {
        sys::mpt_get_send_context_hash(
            into_sys_account(sender),
            into_sys_issuance(issuance),
            sequence,
            into_sys_account(destination),
            version,
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ContextHash::new(out))
}

/// Context hash for `ConfidentialMPTClawback`. Issuer-signed; binds to the
/// holder being clawed back.
pub fn clawback(
    issuer: &AccountId,
    issuance: &IssuanceId,
    sequence: u32,
    holder: &AccountId,
) -> Result<ContextHash> {
    let mut out = [0u8; 32];
    // SAFETY: see `convert`.
    let rc = unsafe {
        sys::mpt_get_clawback_context_hash(
            into_sys_account(issuer),
            into_sys_issuance(issuance),
            sequence,
            into_sys_account(holder),
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ContextHash::new(out))
}
