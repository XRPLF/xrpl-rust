//! High-level assembly of Confidential MPT (XLS-0096) transactions.
//!
//! The [`mpt_crypto`](crate::mpt_crypto) crate exposes the cryptographic
//! primitives (keypairs, ElGamal encrypt/decrypt, Pedersen commitments, context
//! hashes, and the four proof generators). This module wraps them into the
//! full flow a client needs — encrypt → commit → context-hash → prove →
//! populate the model — returning a ready-to-sign `ConfidentialMPT*`
//! transaction. It mirrors xrpl-py's `xrpl.ext.confidential.transaction_builders`
//! _assemble_ layer.
//!
//! These functions are **pure**: the caller supplies mutable ledger-derived
//! state (the account `Sequence`, and for Send/ConvertBack the on-ledger
//! `ConfidentialBalanceSpending` ciphertext + `ConfidentialBalanceVersion` +
//! the decrypted current balance) rather than a client. Read that state with
//! the existing request models, then call the matching assembler.
//!
//! The returned transaction pins `Sequence`: every proof's context hash is
//! bound to that exact value, so autofill must not substitute a different one.
//!
//! Requires the `confidential-mpt` feature.

use alloc::borrow::Cow;
use alloc::string::{String, ToString};

use thiserror_no_std::Error;

use crate::core::addresscodec::decode_classic_address;
use crate::models::transactions::confidential_mpt_clawback::ConfidentialMPTClawback;
use crate::models::transactions::confidential_mpt_convert::ConfidentialMPTConvert;
use crate::models::transactions::confidential_mpt_convert_back::ConfidentialMPTConvertBack;
use crate::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
use crate::models::transactions::confidential_mpt_send::ConfidentialMPTSend;
use crate::models::transactions::{CommonFields, TransactionType};
use crate::models::NoFlags;
use crate::mpt_crypto::{
    commit, context, encrypt, prove, AccountId, Ciphertext, IssuanceId, Privkey, Pubkey,
};

/// Errors returned while assembling a Confidential MPT transaction.
#[derive(Debug, Error)]
pub enum ConfidentialAssemblyError {
    /// A cryptographic operation (encrypt, commit, prove, …) failed.
    #[error("mpt-crypto error: {0}")]
    Crypto(#[from] crate::mpt_crypto::Error),
    /// The account string is not a decodable classic XRPL address.
    #[error("invalid classic address: {0}")]
    InvalidAddress(String),
    /// The `MPTokenIssuanceID` is not 24 bytes of hex (48 chars).
    #[error("invalid MPTokenIssuanceID (expected 48-char hex): {0}")]
    InvalidIssuanceId(String),
    /// An on-ledger ciphertext blob is not 66 bytes of hex (132 chars).
    #[error("invalid ciphertext (expected 132-char hex): {0}")]
    InvalidCiphertext(String),
    /// A ledger query failed, returned an error, or was missing an expected
    /// field (only produced by the `prepare_confidential_*` client helpers).
    #[error("ledger query error: {0}")]
    Ledger(String),
    /// A Send/ConvertBack spends more than the account's decrypted spending
    /// balance. The range proof over the remainder could not be built otherwise
    /// (the balance would go negative), so reject it up front with a clear error.
    #[error("amount {amount} exceeds confidential spending balance {balance}")]
    InsufficientBalance {
        /// The amount being spent.
        amount: u64,
        /// The decrypted current spending balance.
        balance: u64,
    },
}

type Result<T> = core::result::Result<T, ConfidentialAssemblyError>;

fn account_id(address: &str) -> Result<AccountId> {
    let bytes: [u8; 20] = decode_classic_address(address)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| ConfidentialAssemblyError::InvalidAddress(address.to_string()))?;
    Ok(AccountId::new(bytes))
}

fn issuance_id(hex: &str) -> Result<IssuanceId> {
    let bytes: [u8; 24] = hex::decode(hex)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| ConfidentialAssemblyError::InvalidIssuanceId(hex.to_string()))?;
    Ok(IssuanceId::new(bytes))
}

fn ciphertext_from_hex(hex: &str) -> Result<Ciphertext> {
    let bytes: [u8; 66] = hex::decode(hex)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| ConfidentialAssemblyError::InvalidCiphertext(hex.to_string()))?;
    Ok(Ciphertext::new(bytes))
}

fn upper_hex(bytes: &[u8]) -> String {
    hex::encode_upper(bytes)
}

fn common(
    account: &str,
    tx_type: TransactionType,
    sequence: u32,
) -> CommonFields<'static, NoFlags> {
    CommonFields {
        account: Cow::Owned(account.to_string()),
        transaction_type: tx_type,
        // Pin the sequence: the proof's context hash is bound to it.
        sequence: Some(sequence),
        ..Default::default()
    }
}

/// Decrypt an on-ledger ElGamal balance blob (66-byte `c1||c2` hex, e.g.
/// `ConfidentialBalanceSpending`) with the holder's private key.
pub fn decrypt_balance(ciphertext_hex: &str, privkey: &Privkey) -> Result<u64> {
    Ok(encrypt::decrypt(
        &ciphertext_from_hex(ciphertext_hex)?,
        privkey,
    )?)
}

/// Decrypt an on-ledger balance blob, bounding the discrete-log search to
/// `[range_low, range_high]` (cheaper than the unbounded [`decrypt_balance`]
/// when a tight upper bound — e.g. the issuance's outstanding amount — is known).
pub fn decrypt_balance_in_range(
    ciphertext_hex: &str,
    privkey: &Privkey,
    range_low: u64,
    range_high: u64,
) -> Result<u64> {
    Ok(encrypt::decrypt_in_range(
        &ciphertext_from_hex(ciphertext_hex)?,
        privkey,
        range_low,
        range_high,
    )?)
}

/// Inputs for [`assemble_convert`].
pub struct ConvertParams<'a> {
    /// The converting holder's classic address.
    pub account: &'a str,
    /// 48-char hex `MPTokenIssuanceID`.
    pub issuance_id_hex: &'a str,
    /// The account `Sequence` this transaction will be submitted with.
    pub sequence: u32,
    /// Public amount to convert into confidential form.
    pub amount: u64,
    /// The issuer's ElGamal public key (for the issuer mirror ciphertext).
    pub issuer_pubkey: &'a Pubkey,
    /// The holder's ElGamal keypair.
    pub holder_privkey: &'a Privkey,
    pub holder_pubkey: &'a Pubkey,
    /// The optional auditor's ElGamal public key. Required iff the issuance has
    /// an `AuditorEncryptionKey` registered.
    pub auditor_pubkey: Option<&'a Pubkey>,
    /// `true` for the first convert (registers the holder key + Schnorr PoK);
    /// `false` for subsequent converts (rippled returns `tecDUPLICATE` if the
    /// key is re-registered).
    pub register_key: bool,
}

/// Assemble a `ConfidentialMPTConvert` (public → confidential).
pub fn assemble_convert(p: ConvertParams<'_>) -> Result<ConfidentialMPTConvert<'static>> {
    let r = encrypt::random_blinding_factor()?;
    let holder_ct = encrypt::encrypt(p.amount, p.holder_pubkey, &r)?;
    let issuer_ct = encrypt::encrypt(p.amount, p.issuer_pubkey, &r)?;
    let auditor_ct = p
        .auditor_pubkey
        .map(|pk| encrypt::encrypt(p.amount, pk, &r))
        .transpose()?;

    // The holder key + Schnorr proof of knowledge are included only on the first
    // convert (the opt-in). Subsequent converts omit both.
    let (holder_encryption_key, zk_proof) = if p.register_key {
        let ctx = context::convert(
            &account_id(p.account)?,
            &issuance_id(p.issuance_id_hex)?,
            p.sequence,
        )?;
        let proof = prove::convert(p.holder_privkey, p.holder_pubkey, &ctx)?;
        (
            Some(Cow::Owned(upper_hex(p.holder_pubkey.as_bytes()))),
            Some(Cow::Owned(upper_hex(proof.as_bytes()))),
        )
    } else {
        (None, None)
    };

    Ok(ConfidentialMPTConvert {
        common_fields: common(
            p.account,
            TransactionType::ConfidentialMPTConvert,
            p.sequence,
        ),
        mptoken_issuance_id: Cow::Owned(p.issuance_id_hex.to_string()),
        mpt_amount: Cow::Owned(p.amount.to_string()),
        holder_encrypted_amount: Cow::Owned(upper_hex(holder_ct.as_bytes())),
        issuer_encrypted_amount: Cow::Owned(upper_hex(issuer_ct.as_bytes())),
        blinding_factor: Cow::Owned(upper_hex(r.as_bytes())),
        holder_encryption_key,
        auditor_encrypted_amount: auditor_ct.map(|ct| Cow::Owned(upper_hex(ct.as_bytes()))),
        zk_proof,
    })
}

/// Inputs for [`assemble_send`].
pub struct SendParams<'a> {
    /// The sender's classic address.
    pub sender_account: &'a str,
    /// The receiver's classic address.
    pub destination_account: &'a str,
    /// Optional `DestinationTag` (e.g. for a hosted/exchange receiver).
    pub destination_tag: Option<u32>,
    /// 48-char hex `MPTokenIssuanceID`.
    pub issuance_id_hex: &'a str,
    /// The sender's account `Sequence`.
    pub sequence: u32,
    /// The sender's on-ledger `ConfidentialBalanceVersion`.
    pub version: u32,
    /// The confidential amount to send.
    pub amount: u64,
    /// The sender's decrypted current spending balance.
    pub current_balance: u64,
    /// The sender's on-ledger `ConfidentialBalanceSpending` (132-char hex).
    pub balance_ciphertext_hex: &'a str,
    pub sender_privkey: &'a Privkey,
    pub sender_pubkey: &'a Pubkey,
    pub destination_pubkey: &'a Pubkey,
    pub issuer_pubkey: &'a Pubkey,
    pub auditor_pubkey: Option<&'a Pubkey>,
    /// XLS-70 `CredentialIDs` presented to satisfy the destination's
    /// `DepositPreauth` / credential-based authorization, if it requires one.
    /// Each entry is a credential's 64-char hex ledger index; `None` (or an empty
    /// slice) omits the field.
    pub credential_ids: Option<&'a [&'a str]>,
}

/// Assemble a `ConfidentialMPTSend` (confidential transfer).
pub fn assemble_send(p: SendParams<'_>) -> Result<ConfidentialMPTSend<'static>> {
    if p.amount > p.current_balance {
        return Err(ConfidentialAssemblyError::InsufficientBalance {
            amount: p.amount,
            balance: p.current_balance,
        });
    }
    let tx_r = encrypt::random_blinding_factor()?;
    let sender_ct = encrypt::encrypt(p.amount, p.sender_pubkey, &tx_r)?;
    let dest_ct = encrypt::encrypt(p.amount, p.destination_pubkey, &tx_r)?;
    let issuer_ct = encrypt::encrypt(p.amount, p.issuer_pubkey, &tx_r)?;
    let auditor_ct = p
        .auditor_pubkey
        .map(|pk| encrypt::encrypt(p.amount, pk, &tx_r))
        .transpose()?;
    let amount_commitment = commit::pedersen(p.amount, &tx_r)?;

    let balance_blinding = encrypt::random_blinding_factor()?;
    let balance_commitment = commit::pedersen(p.current_balance, &balance_blinding)?;
    let balance_ciphertext = ciphertext_from_hex(p.balance_ciphertext_hex)?;

    let ctx = context::send(
        &account_id(p.sender_account)?,
        &issuance_id(p.issuance_id_hex)?,
        p.sequence,
        &account_id(p.destination_account)?,
        p.version,
    )?;

    let auditor_participant = match (p.auditor_pubkey, &auditor_ct) {
        (Some(pk), Some(ct)) => Some(prove::Participant {
            pubkey: pk,
            ciphertext: ct,
        }),
        _ => None,
    };
    let proof = prove::send(prove::SendProofParams {
        sender_privkey: p.sender_privkey,
        sender_pubkey: p.sender_pubkey,
        amount: p.amount,
        current_balance: p.current_balance,
        tx_blinding_factor: &tx_r,
        context_hash: &ctx,
        amount_commitment: &amount_commitment,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &balance_ciphertext,
        sender: prove::Participant {
            pubkey: p.sender_pubkey,
            ciphertext: &sender_ct,
        },
        destination: prove::Participant {
            pubkey: p.destination_pubkey,
            ciphertext: &dest_ct,
        },
        issuer: prove::Participant {
            pubkey: p.issuer_pubkey,
            ciphertext: &issuer_ct,
        },
        auditor: auditor_participant,
    })?;

    Ok(ConfidentialMPTSend {
        common_fields: common(
            p.sender_account,
            TransactionType::ConfidentialMPTSend,
            p.sequence,
        ),
        destination: Cow::Owned(p.destination_account.to_string()),
        destination_tag: p.destination_tag,
        mptoken_issuance_id: Cow::Owned(p.issuance_id_hex.to_string()),
        sender_encrypted_amount: Cow::Owned(upper_hex(sender_ct.as_bytes())),
        destination_encrypted_amount: Cow::Owned(upper_hex(dest_ct.as_bytes())),
        issuer_encrypted_amount: Cow::Owned(upper_hex(issuer_ct.as_bytes())),
        amount_commitment: Cow::Owned(upper_hex(amount_commitment.as_bytes())),
        balance_commitment: Cow::Owned(upper_hex(balance_commitment.as_bytes())),
        zk_proof: Cow::Owned(upper_hex(proof.as_bytes())),
        auditor_encrypted_amount: auditor_ct.map(|ct| Cow::Owned(upper_hex(ct.as_bytes()))),
        credential_ids: p.credential_ids.map(|ids| {
            ids.iter()
                .map(|id| Cow::Owned(id.to_string()))
                .collect::<Vec<_>>()
        }),
    })
}

/// Inputs for [`assemble_convert_back`].
pub struct ConvertBackParams<'a> {
    /// The holder's classic address.
    pub account: &'a str,
    /// 48-char hex `MPTokenIssuanceID`.
    pub issuance_id_hex: &'a str,
    /// The holder's account `Sequence`.
    pub sequence: u32,
    /// The holder's on-ledger `ConfidentialBalanceVersion`.
    pub version: u32,
    /// The confidential amount to convert back to public.
    pub amount: u64,
    /// The holder's decrypted current spending balance.
    pub current_balance: u64,
    /// The holder's on-ledger `ConfidentialBalanceSpending` (132-char hex).
    pub balance_ciphertext_hex: &'a str,
    pub holder_privkey: &'a Privkey,
    pub holder_pubkey: &'a Pubkey,
    pub issuer_pubkey: &'a Pubkey,
    pub auditor_pubkey: Option<&'a Pubkey>,
}

/// Assemble a `ConfidentialMPTConvertBack` (confidential → public).
pub fn assemble_convert_back(
    p: ConvertBackParams<'_>,
) -> Result<ConfidentialMPTConvertBack<'static>> {
    if p.amount > p.current_balance {
        return Err(ConfidentialAssemblyError::InsufficientBalance {
            amount: p.amount,
            balance: p.current_balance,
        });
    }
    let r = encrypt::random_blinding_factor()?;
    let holder_ct = encrypt::encrypt(p.amount, p.holder_pubkey, &r)?;
    let issuer_ct = encrypt::encrypt(p.amount, p.issuer_pubkey, &r)?;
    let auditor_ct = p
        .auditor_pubkey
        .map(|pk| encrypt::encrypt(p.amount, pk, &r))
        .transpose()?;

    let balance_blinding = encrypt::random_blinding_factor()?;
    let balance_commitment = commit::pedersen(p.current_balance, &balance_blinding)?;
    let balance_ciphertext = ciphertext_from_hex(p.balance_ciphertext_hex)?;

    let ctx = context::convert_back(
        &account_id(p.account)?,
        &issuance_id(p.issuance_id_hex)?,
        p.sequence,
        p.version,
    )?;
    let proof = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey: p.holder_privkey,
        holder_pubkey: p.holder_pubkey,
        amount: p.amount,
        current_balance: p.current_balance,
        context_hash: &ctx,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &balance_ciphertext,
    })?;

    Ok(ConfidentialMPTConvertBack {
        common_fields: common(
            p.account,
            TransactionType::ConfidentialMPTConvertBack,
            p.sequence,
        ),
        mptoken_issuance_id: Cow::Owned(p.issuance_id_hex.to_string()),
        mpt_amount: Cow::Owned(p.amount.to_string()),
        holder_encrypted_amount: Cow::Owned(upper_hex(holder_ct.as_bytes())),
        issuer_encrypted_amount: Cow::Owned(upper_hex(issuer_ct.as_bytes())),
        blinding_factor: Cow::Owned(upper_hex(r.as_bytes())),
        balance_commitment: Cow::Owned(upper_hex(balance_commitment.as_bytes())),
        zk_proof: Cow::Owned(upper_hex(proof.as_bytes())),
        auditor_encrypted_amount: auditor_ct.map(|ct| Cow::Owned(upper_hex(ct.as_bytes()))),
    })
}

/// Inputs for [`assemble_clawback`].
pub struct ClawbackParams<'a> {
    /// The issuer's classic address (transaction sender).
    pub issuer_account: &'a str,
    /// The holder whose balance is being clawed back.
    pub holder_account: &'a str,
    /// 48-char hex `MPTokenIssuanceID`.
    pub issuance_id_hex: &'a str,
    /// The issuer's account `Sequence`.
    pub sequence: u32,
    /// The plaintext amount being reclaimed.
    pub amount: u64,
    pub issuer_privkey: &'a Privkey,
    pub issuer_pubkey: &'a Pubkey,
    /// The holder's on-ledger `IssuerEncryptedBalance` (132-char hex).
    pub issuer_encrypted_balance_hex: &'a str,
}

/// Assemble a `ConfidentialMPTClawback` (issuer reclaims a holder's balance).
pub fn assemble_clawback(p: ClawbackParams<'_>) -> Result<ConfidentialMPTClawback<'static>> {
    let ctx = context::clawback(
        &account_id(p.issuer_account)?,
        &issuance_id(p.issuance_id_hex)?,
        p.sequence,
        &account_id(p.holder_account)?,
    )?;
    let proof = prove::clawback(
        p.issuer_privkey,
        p.issuer_pubkey,
        &ctx,
        p.amount,
        &ciphertext_from_hex(p.issuer_encrypted_balance_hex)?,
    )?;

    Ok(ConfidentialMPTClawback {
        common_fields: common(
            p.issuer_account,
            TransactionType::ConfidentialMPTClawback,
            p.sequence,
        ),
        holder: Cow::Owned(p.holder_account.to_string()),
        mptoken_issuance_id: Cow::Owned(p.issuance_id_hex.to_string()),
        mpt_amount: Cow::Owned(p.amount.to_string()),
        zk_proof: Cow::Owned(upper_hex(proof.as_bytes())),
    })
}

/// Assemble a `ConfidentialMPTMergeInbox` (merge inbox into spending balance).
/// No cryptographic material is required.
pub fn assemble_merge_inbox(
    account: &str,
    issuance_id_hex: &str,
    sequence: u32,
) -> ConfidentialMPTMergeInbox<'static> {
    ConfidentialMPTMergeInbox {
        common_fields: common(
            account,
            TransactionType::ConfidentialMPTMergeInbox,
            sequence,
        ),
        mptoken_issuance_id: Cow::Owned(issuance_id_hex.to_string()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client-querying convenience layer.
//
// The `assemble_*` functions above are pure — the caller supplies all
// ledger-derived state. These `prepare_confidential_*` wrappers fetch that state
// from an async client (account sequence, the on-ledger MPToken's confidential
// balance ciphertext + version, decrypting the current balance), mirroring
// xrpl-py's `prepare_confidential_*` layer. The fee is left to autofill.
//
// This layer needs `asynch::account` (`helpers`) and the `XRPLAsyncClient` trait
// in `asynch::clients` (`json-rpc`/`websocket`), plus a runtime for the retry
// sleeps. Those aren't part of `confidential-mpt` itself (they'd force a runtime
// choice on the caller), so gate the module on them — the pure `assemble_*`
// layer above stays usable with `confidential-mpt` alone.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(all(feature = "helpers", any(feature = "json-rpc", feature = "websocket")))]
pub use prepare::*;

#[cfg(all(feature = "helpers", any(feature = "json-rpc", feature = "websocket")))]
mod prepare {
    use alloc::format;
    use alloc::string::ToString;

    use serde_json::Value;

    use super::{
        assemble_clawback, assemble_convert, assemble_convert_back, assemble_merge_inbox,
        assemble_send, decrypt_balance_in_range, ClawbackParams, ConfidentialAssemblyError,
        ConvertBackParams, ConvertParams, Result, SendParams,
    };
    use crate::asynch::account::get_next_valid_seq_number;
    use crate::asynch::clients::XRPLAsyncClient;
    use crate::models::requests::account_objects::{AccountObjectType, AccountObjects};
    use crate::models::requests::{CommonFields, Marker, RequestMethod};
    use crate::models::results::account_objects::AccountObjects as AccountObjectsResult;
    use crate::models::transactions::confidential_mpt_clawback::ConfidentialMPTClawback;
    use crate::models::transactions::confidential_mpt_convert::ConfidentialMPTConvert;
    use crate::models::transactions::confidential_mpt_convert_back::ConfidentialMPTConvertBack;
    use crate::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
    use crate::models::transactions::confidential_mpt_send::ConfidentialMPTSend;
    use crate::mpt_crypto::{Privkey, Pubkey};

    async fn fetch_sequence<C: XRPLAsyncClient>(client: &C, account: &str) -> Result<u32> {
        get_next_valid_seq_number(account.to_string().into(), client, None)
            .await
            .map_err(|e| ConfidentialAssemblyError::Ledger(format!("fetch sequence failed: {e}")))
    }

    /// Read `account`'s MPToken for `issuance_id_hex` as raw JSON (the typed
    /// ledger objects do not yet expose the confidential fields).
    async fn read_mptoken<C: XRPLAsyncClient>(
        client: &C,
        account: &str,
        issuance_id_hex: &str,
    ) -> Result<Value> {
        // Page through account_objects following `marker`: an account with many
        // objects returns them across pages, so the MPToken we want may not be
        // on the first page. Own the marker (into_owned) so it survives past the
        // response it came from, into the next request.
        let mut marker: Option<Marker<'static>> = None;
        loop {
            let request = AccountObjects {
                common_fields: CommonFields {
                    command: RequestMethod::AccountObjects,
                    id: None,
                },
                account: account.to_string().into(),
                ledger_lookup: None,
                r#type: Some(AccountObjectType::Mptoken),
                deletion_blockers_only: None,
                limit: None,
                marker,
            };
            let response = client.request(request.into()).await.map_err(|e| {
                ConfidentialAssemblyError::Ledger(format!("account_objects request failed: {e}"))
            })?;
            let objects = AccountObjectsResult::try_from(response).map_err(|e| {
                ConfidentialAssemblyError::Ledger(format!("could not parse account_objects: {e}"))
            })?;
            // Case-insensitive: the ledger returns uppercase hex, but a caller
            // may pass lowercase (e.g. from `hex::encode`); the rest of this
            // module parses issuance IDs case-insensitively via `hex::decode`.
            if let Some(obj) = objects.account_objects.iter().find(|o| {
                o.get("MPTokenIssuanceID")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.eq_ignore_ascii_case(issuance_id_hex))
            }) {
                return Ok(obj.clone());
            }
            match objects.marker {
                Some(Marker::Str(s)) => marker = Some(Marker::Str(s.into_owned().into())),
                Some(Marker::Int(i)) => marker = Some(Marker::Int(i)),
                Some(Marker::Sequence(sq)) => marker = Some(Marker::Sequence(sq)),
                None => {
                    return Err(ConfidentialAssemblyError::Ledger(format!(
                        "no MPToken for issuance {issuance_id_hex} owned by {account}"
                    )))
                }
            }
        }
    }

    fn field_str<'a>(node: &'a Value, field: &str) -> Result<&'a str> {
        node.get(field).and_then(Value::as_str).ok_or_else(|| {
            ConfidentialAssemblyError::Ledger(format!("MPToken is missing field {field}"))
        })
    }

    fn balance_version(node: &Value) -> u32 {
        node.get("ConfidentialBalanceVersion")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32
    }

    /// Prepare a `ConfidentialMPTConvert`, fetching the account sequence and
    /// auto-detecting whether this is the first convert (which registers the
    /// holder key) from the on-ledger MPToken.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_confidential_convert<C: XRPLAsyncClient>(
        client: &C,
        account: &str,
        issuance_id_hex: &str,
        amount: u64,
        issuer_pubkey: &Pubkey,
        holder_privkey: &Privkey,
        holder_pubkey: &Pubkey,
        auditor_pubkey: Option<&Pubkey>,
    ) -> Result<ConfidentialMPTConvert<'static>> {
        let sequence = fetch_sequence(client, account).await?;
        let node = read_mptoken(client, account, issuance_id_hex).await?;
        // First convert (opt-in) registers the holder key; later ones must not
        // (rippled returns tecDUPLICATE).
        let register_key = node
            .get("HolderEncryptionKey")
            .and_then(Value::as_str)
            .is_none();
        assemble_convert(ConvertParams {
            account,
            issuance_id_hex,
            sequence,
            amount,
            issuer_pubkey,
            holder_privkey,
            holder_pubkey,
            auditor_pubkey,
            register_key,
        })
    }

    /// Prepare a `ConfidentialMPTSend`, fetching the sender's sequence + on-ledger
    /// spending balance and decrypting it. `max_balance` bounds the decrypt
    /// discrete-log search (cost is O(max_balance)); pass the issuance's
    /// outstanding amount or a known upper bound.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_confidential_send<C: XRPLAsyncClient>(
        client: &C,
        sender_account: &str,
        destination_account: &str,
        destination_tag: Option<u32>,
        issuance_id_hex: &str,
        amount: u64,
        max_balance: u64,
        sender_privkey: &Privkey,
        sender_pubkey: &Pubkey,
        destination_pubkey: &Pubkey,
        issuer_pubkey: &Pubkey,
        auditor_pubkey: Option<&Pubkey>,
        credential_ids: Option<&[&str]>,
    ) -> Result<ConfidentialMPTSend<'static>> {
        let sequence = fetch_sequence(client, sender_account).await?;
        let node = read_mptoken(client, sender_account, issuance_id_hex).await?;
        let balance_ciphertext_hex = field_str(&node, "ConfidentialBalanceSpending")?.to_string();
        let version = balance_version(&node);
        let current_balance =
            decrypt_balance_in_range(&balance_ciphertext_hex, sender_privkey, 0, max_balance)?;
        assemble_send(SendParams {
            sender_account,
            destination_account,
            destination_tag,
            issuance_id_hex,
            sequence,
            version,
            amount,
            current_balance,
            balance_ciphertext_hex: &balance_ciphertext_hex,
            sender_privkey,
            sender_pubkey,
            destination_pubkey,
            issuer_pubkey,
            auditor_pubkey,
            credential_ids,
        })
    }

    /// Prepare a `ConfidentialMPTConvertBack`, fetching + decrypting the holder's
    /// on-ledger spending balance. See `prepare_confidential_send` re `max_balance`.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_confidential_convert_back<C: XRPLAsyncClient>(
        client: &C,
        account: &str,
        issuance_id_hex: &str,
        amount: u64,
        max_balance: u64,
        holder_privkey: &Privkey,
        holder_pubkey: &Pubkey,
        issuer_pubkey: &Pubkey,
        auditor_pubkey: Option<&Pubkey>,
    ) -> Result<ConfidentialMPTConvertBack<'static>> {
        let sequence = fetch_sequence(client, account).await?;
        let node = read_mptoken(client, account, issuance_id_hex).await?;
        let balance_ciphertext_hex = field_str(&node, "ConfidentialBalanceSpending")?.to_string();
        let version = balance_version(&node);
        let current_balance =
            decrypt_balance_in_range(&balance_ciphertext_hex, holder_privkey, 0, max_balance)?;
        assemble_convert_back(ConvertBackParams {
            account,
            issuance_id_hex,
            sequence,
            version,
            amount,
            current_balance,
            balance_ciphertext_hex: &balance_ciphertext_hex,
            holder_privkey,
            holder_pubkey,
            issuer_pubkey,
            auditor_pubkey,
        })
    }

    /// Prepare a `ConfidentialMPTClawback`, fetching the issuer's sequence and the
    /// holder's on-ledger `IssuerEncryptedBalance`.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_confidential_clawback<C: XRPLAsyncClient>(
        client: &C,
        issuer_account: &str,
        holder_account: &str,
        issuance_id_hex: &str,
        amount: u64,
        issuer_privkey: &Privkey,
        issuer_pubkey: &Pubkey,
    ) -> Result<ConfidentialMPTClawback<'static>> {
        let sequence = fetch_sequence(client, issuer_account).await?;
        let node = read_mptoken(client, holder_account, issuance_id_hex).await?;
        let issuer_encrypted_balance_hex = field_str(&node, "IssuerEncryptedBalance")?.to_string();
        assemble_clawback(ClawbackParams {
            issuer_account,
            holder_account,
            issuance_id_hex,
            sequence,
            amount,
            issuer_privkey,
            issuer_pubkey,
            issuer_encrypted_balance_hex: &issuer_encrypted_balance_hex,
        })
    }

    /// Prepare a `ConfidentialMPTMergeInbox`, fetching the account sequence.
    pub async fn prepare_confidential_merge_inbox<C: XRPLAsyncClient>(
        client: &C,
        account: &str,
        issuance_id_hex: &str,
    ) -> Result<ConfidentialMPTMergeInbox<'static>> {
        let sequence = fetch_sequence(client, account).await?;
        Ok(assemble_merge_inbox(account, issuance_id_hex, sequence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;
    use crate::mpt_crypto::keypair;

    const ACCOUNT: &str = "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW";
    const ISSUANCE: &str = "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47";

    #[test]
    fn decrypt_balance_roundtrip() {
        let (sk, pk) = keypair::generate().unwrap();
        let r = encrypt::random_blinding_factor().unwrap();
        let ct = encrypt::encrypt(4242, &pk, &r).unwrap();
        assert_eq!(
            decrypt_balance(&upper_hex(ct.as_bytes()), &sk).unwrap(),
            4242
        );
    }

    #[test]
    fn convert_first_registers_key_and_validates() {
        let (_issuer_sk, issuer_pk) = keypair::generate().unwrap();
        let (holder_sk, holder_pk) = keypair::generate().unwrap();
        let tx = assemble_convert(ConvertParams {
            account: ACCOUNT,
            issuance_id_hex: ISSUANCE,
            sequence: 5,
            amount: 1000,
            issuer_pubkey: &issuer_pk,
            holder_privkey: &holder_sk,
            holder_pubkey: &holder_pk,
            auditor_pubkey: None,
            register_key: true,
        })
        .unwrap();
        assert!(tx.holder_encryption_key.is_some());
        assert!(tx.zk_proof.is_some());
        assert_eq!(tx.common_fields.sequence, Some(5));
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn convert_subsequent_omits_key_and_proof() {
        let (_issuer_sk, issuer_pk) = keypair::generate().unwrap();
        let (holder_sk, holder_pk) = keypair::generate().unwrap();
        let tx = assemble_convert(ConvertParams {
            account: ACCOUNT,
            issuance_id_hex: ISSUANCE,
            sequence: 6,
            amount: 1000,
            issuer_pubkey: &issuer_pk,
            holder_privkey: &holder_sk,
            holder_pubkey: &holder_pk,
            auditor_pubkey: None,
            register_key: false,
        })
        .unwrap();
        assert!(tx.holder_encryption_key.is_none());
        assert!(tx.zk_proof.is_none());
    }

    #[test]
    fn convert_with_auditor_sets_auditor_ciphertext() {
        let (_issuer_sk, issuer_pk) = keypair::generate().unwrap();
        let (holder_sk, holder_pk) = keypair::generate().unwrap();
        let (_auditor_sk, auditor_pk) = keypair::generate().unwrap();
        let tx = assemble_convert(ConvertParams {
            account: ACCOUNT,
            issuance_id_hex: ISSUANCE,
            sequence: 5,
            amount: 1000,
            issuer_pubkey: &issuer_pk,
            holder_privkey: &holder_sk,
            holder_pubkey: &holder_pk,
            auditor_pubkey: Some(&auditor_pk),
            register_key: true,
        })
        .unwrap();
        assert!(tx.auditor_encrypted_amount.is_some());
    }

    #[test]
    fn merge_inbox_builds_and_validates() {
        let tx = assemble_merge_inbox(ACCOUNT, ISSUANCE, 7);
        assert_eq!(tx.common_fields.sequence, Some(7));
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn invalid_address_is_rejected() {
        let (_sk, pk) = keypair::generate().unwrap();
        let (holder_sk, holder_pk) = keypair::generate().unwrap();
        let err = assemble_convert(ConvertParams {
            account: "not_a_valid_address",
            issuance_id_hex: ISSUANCE,
            sequence: 5,
            amount: 1000,
            issuer_pubkey: &pk,
            holder_privkey: &holder_sk,
            holder_pubkey: &holder_pk,
            auditor_pubkey: None,
            register_key: true,
        });
        assert!(matches!(
            err,
            Err(ConfidentialAssemblyError::InvalidAddress(_))
        ));
    }

    #[test]
    fn assemble_send_rejects_overspend() {
        let (sk, pk) = keypair::generate().unwrap();
        let (_dsk, dpk) = keypair::generate().unwrap();
        // amount (100) > current_balance (50): rejected before any proof work
        // (the check runs first, so the ciphertext hex is never parsed).
        let err = assemble_send(SendParams {
            sender_account: ACCOUNT,
            destination_account: ACCOUNT,
            destination_tag: None,
            issuance_id_hex: ISSUANCE,
            sequence: 1,
            version: 0,
            amount: 100,
            current_balance: 50,
            balance_ciphertext_hex: "",
            sender_privkey: &sk,
            sender_pubkey: &pk,
            destination_pubkey: &dpk,
            issuer_pubkey: &pk,
            auditor_pubkey: None,
            credential_ids: None,
        })
        .unwrap_err();
        assert!(matches!(
            err,
            ConfidentialAssemblyError::InsufficientBalance {
                amount: 100,
                balance: 50
            }
        ));
    }

    #[test]
    fn send_threads_credential_ids() {
        // XLS-70 CredentialIDs supplied on SendParams are carried onto the tx.
        let (sk, pk) = keypair::generate().unwrap();
        let (_dsk, dpk) = keypair::generate().unwrap();
        let (_isk, ipk) = keypair::generate().unwrap();
        let r = encrypt::random_blinding_factor().unwrap();
        let balance_hex = upper_hex(encrypt::encrypt(1000, &pk, &r).unwrap().as_bytes());
        let cred_a = "AB".repeat(32); // 64-hex credential ledger index
        let cred_b = "CD".repeat(32);
        let creds = [cred_a.as_str(), cred_b.as_str()];

        let tx = assemble_send(SendParams {
            sender_account: ACCOUNT,
            destination_account: ACCOUNT,
            destination_tag: None,
            issuance_id_hex: ISSUANCE,
            sequence: 1,
            version: 0,
            amount: 10,
            current_balance: 1000,
            balance_ciphertext_hex: &balance_hex,
            sender_privkey: &sk,
            sender_pubkey: &pk,
            destination_pubkey: &dpk,
            issuer_pubkey: &ipk,
            auditor_pubkey: None,
            credential_ids: Some(&creds),
        })
        .unwrap();

        let got = tx.credential_ids.expect("credential_ids threaded through");
        assert_eq!(got.len(), 2);
        assert_eq!(got[0], cred_a);
        assert_eq!(got[1], cred_b);
    }

    #[test]
    fn assemble_send_happy_path() {
        // A fully-formed send: correct field shapes, the destination ciphertext
        // decrypts to the sent amount, and the assembled model validates.
        let (sk, pk) = keypair::generate().unwrap();
        let (dsk, dpk) = keypair::generate().unwrap();
        let (_isk, ipk) = keypair::generate().unwrap();
        let r = encrypt::random_blinding_factor().unwrap();
        let balance_hex = upper_hex(encrypt::encrypt(1000, &pk, &r).unwrap().as_bytes());

        // A second real address, distinct from the sender and from the issuer
        // embedded in ISSUANCE, so the self-send / issuer-role bans pass.
        const DESTINATION: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let tx = assemble_send(SendParams {
            sender_account: ACCOUNT,
            destination_account: DESTINATION,
            destination_tag: Some(42),
            issuance_id_hex: ISSUANCE,
            sequence: 7,
            version: 3,
            amount: 250,
            current_balance: 1000,
            balance_ciphertext_hex: &balance_hex,
            sender_privkey: &sk,
            sender_pubkey: &pk,
            destination_pubkey: &dpk,
            issuer_pubkey: &ipk,
            auditor_pubkey: None,
            credential_ids: None,
        })
        .unwrap();

        assert_eq!(tx.destination.as_ref(), DESTINATION);
        assert_eq!(tx.destination_tag, Some(42));
        assert_eq!(tx.common_fields.sequence, Some(7));
        assert_eq!(tx.mptoken_issuance_id.as_ref(), ISSUANCE);
        // ElGamal ciphertexts are 132 hex chars; Pedersen commitments 66; the
        // composite Send proof is 946 bytes = 1892 hex chars.
        assert_eq!(tx.sender_encrypted_amount.len(), 132);
        assert_eq!(tx.destination_encrypted_amount.len(), 132);
        assert_eq!(tx.issuer_encrypted_amount.len(), 132);
        assert_eq!(tx.amount_commitment.len(), 66);
        assert_eq!(tx.balance_commitment.len(), 66);
        assert_eq!(tx.zk_proof.len(), 1892);
        assert!(tx.auditor_encrypted_amount.is_none());

        // The destination ciphertext really encrypts the sent amount.
        assert_eq!(
            decrypt_balance(tx.destination_encrypted_amount.as_ref(), &dsk).unwrap(),
            250
        );
        // The assembled transaction is a valid model.
        assert!(tx.validate().is_ok());
    }
}
