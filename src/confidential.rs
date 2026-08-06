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
}

/// Assemble a `ConfidentialMPTSend` (confidential transfer).
pub fn assemble_send(p: SendParams<'_>) -> Result<ConfidentialMPTSend<'static>> {
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
        destination_tag: None,
        mptoken_issuance_id: Cow::Owned(p.issuance_id_hex.to_string()),
        sender_encrypted_amount: Cow::Owned(upper_hex(sender_ct.as_bytes())),
        destination_encrypted_amount: Cow::Owned(upper_hex(dest_ct.as_bytes())),
        issuer_encrypted_amount: Cow::Owned(upper_hex(issuer_ct.as_bytes())),
        amount_commitment: Cow::Owned(upper_hex(amount_commitment.as_bytes())),
        balance_commitment: Cow::Owned(upper_hex(balance_commitment.as_bytes())),
        zk_proof: Cow::Owned(upper_hex(proof.as_bytes())),
        auditor_encrypted_amount: auditor_ct.map(|ct| Cow::Owned(upper_hex(ct.as_bytes()))),
        credential_ids: None,
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
}
