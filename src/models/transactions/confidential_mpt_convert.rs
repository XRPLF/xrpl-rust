use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies, XRPLModelException,
};
use crate::models::{FlagCollection, NoFlags};

use super::confidential_mpt_constants::{
    validate_hex_length, validate_mpt_amount, BLINDING_FACTOR_LENGTH, CIPHERTEXT_LENGTH,
    ENCRYPTION_KEY_LENGTH, SCHNORR_PROOF_LENGTH,
};
use super::mptoken_issuance_set::validate_mptoken_issuance_id;
use super::{CommonFields, CommonTransactionBuilder};

/// A `ConfidentialMPTConvert` transaction converts a holder's public MPT
/// balance into confidential form (XLS-0096 §7).
///
/// On first use it also serves as the **opt-in** for confidential MPTs: the
/// holder registers their `HolderEncryptionKey` and provides a 64-byte
/// Schnorr Proof of Knowledge of the corresponding secret key.
///
/// On subsequent calls (key already registered) `holder_encryption_key`
/// and `zk_proof` MUST both be absent — those fields are gated by §7.3.1
/// rules 2 and 3.
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct ConfidentialMPTConvert<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    /// 24-byte `MPTokenIssuanceID` of the target MPT.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,

    /// Plaintext amount being converted from public to confidential.
    /// Encoded as a u64 string per XRPL's large-integer convention.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext credited to the holder's `CB_IN`.
    pub holder_encrypted_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext credited to the issuer's mirror balance.
    pub issuer_encrypted_amount: Cow<'a, str>,

    /// 32-byte ElGamal randomness `r`. Revealed plaintext so validators
    /// can deterministically verify the ciphertexts encrypt `mpt_amount`.
    pub blinding_factor: Cow<'a, str>,

    /// 33-byte compressed holder ElGamal public key. **Required** on first
    /// Convert (key registration); **forbidden** thereafter.
    pub holder_encryption_key: Option<Cow<'a, str>>,

    /// 66-byte ElGamal ciphertext for the auditor mirror. Required iff the
    /// issuance has an `AuditorEncryptionKey` registered.
    pub auditor_encrypted_amount: Option<Cow<'a, str>>,

    /// 64-byte Schnorr Proof of Knowledge of the holder's secret key.
    /// **Required** if `holder_encryption_key` is present; **forbidden**
    /// otherwise.
    #[serde(rename = "ZKProof")]
    pub zk_proof: Option<Cow<'a, str>>,
}

impl<'a> Model for ConfidentialMPTConvert<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self._get_registration_error()?;
        self._get_field_length_errors()?;
        self.validate_currencies()
    }
}

impl<'a> ConfidentialMPTConvert<'a> {
    /// `HolderEncryptionKey` and the Schnorr `ZKProof` are all-or-nothing:
    /// both present on the registering (first) Convert, both absent after
    /// (XLS-0096 §7.3.1 rules 2 and 3).
    fn _get_registration_error(&self) -> crate::models::XRPLModelResult<()> {
        match (
            self.holder_encryption_key.is_some(),
            self.zk_proof.is_some(),
        ) {
            (true, false) => Err(XRPLModelException::FieldRequiresField {
                field1: "holder_encryption_key".into(),
                field2: "zk_proof".into(),
            }),
            (false, true) => Err(XRPLModelException::FieldRequiresField {
                field1: "zk_proof".into(),
                field2: "holder_encryption_key".into(),
            }),
            _ => Ok(()),
        }
    }

    fn _get_field_length_errors(&self) -> crate::models::XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        validate_mpt_amount("mpt_amount", self.mpt_amount.as_ref(), false)?;
        validate_hex_length(
            "holder_encrypted_amount",
            self.holder_encrypted_amount.as_ref(),
            CIPHERTEXT_LENGTH,
        )?;
        validate_hex_length(
            "issuer_encrypted_amount",
            self.issuer_encrypted_amount.as_ref(),
            CIPHERTEXT_LENGTH,
        )?;
        if let Some(auditor) = self.auditor_encrypted_amount.as_deref() {
            validate_hex_length("auditor_encrypted_amount", auditor, CIPHERTEXT_LENGTH)?;
        }
        validate_hex_length(
            "blinding_factor",
            self.blinding_factor.as_ref(),
            BLINDING_FACTOR_LENGTH,
        )?;
        if let Some(key) = self.holder_encryption_key.as_deref() {
            validate_hex_length("holder_encryption_key", key, ENCRYPTION_KEY_LENGTH)?;
        }
        if let Some(proof) = self.zk_proof.as_deref() {
            validate_hex_length("zk_proof", proof, SCHNORR_PROOF_LENGTH)?;
        }
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTConvert<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTConvert<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTConvert<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        mptoken_issuance_id: Cow<'a, str>,
        mpt_amount: Cow<'a, str>,
        holder_encrypted_amount: Cow<'a, str>,
        issuer_encrypted_amount: Cow<'a, str>,
        blinding_factor: Cow<'a, str>,
        holder_encryption_key: Option<Cow<'a, str>>,
        auditor_encrypted_amount: Option<Cow<'a, str>>,
        zk_proof: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTConvert,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            mptoken_issuance_id,
            mpt_amount,
            holder_encrypted_amount,
            issuer_encrypted_amount,
            blinding_factor,
            holder_encryption_key,
            auditor_encrypted_amount,
            zk_proof,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_serialize_first_convert_with_registration() {
        let tx = ConfidentialMPTConvert {
            common_fields: CommonFields {
                account: "rUserAccount11111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTConvert,
                ..Default::default()
            },
            mptoken_issuance_id: "610F33B8EBF7EC795F822A454FB852156AEFE50BE0CB8326338A81CD74801864"
                .into(),
            mpt_amount: "1000".into(),
            holder_encrypted_amount: "AD3F".repeat(33).into(),
            issuer_encrypted_amount: "BC2E".repeat(33).into(),
            blinding_factor: "EE".repeat(32).into(),
            holder_encryption_key: Some("03".to_string() + &"8d".repeat(32)).map(Into::into),
            auditor_encrypted_amount: None,
            zk_proof: Some("AB".repeat(64).into()),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTConvert\""));
        assert!(json.contains("\"HolderEncryptionKey\""));
        assert!(json.contains("\"ZKProof\""));

        let round_tripped: ConfidentialMPTConvert = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }

    #[test]
    fn test_serialize_subsequent_convert_no_key() {
        let tx = ConfidentialMPTConvert {
            common_fields: CommonFields {
                account: "rUserAccount11111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTConvert,
                ..Default::default()
            },
            mptoken_issuance_id: "610F33".repeat(4).into(),
            mpt_amount: "500".into(),
            holder_encrypted_amount: "AD3F".repeat(33).into(),
            issuer_encrypted_amount: "BC2E".repeat(33).into(),
            blinding_factor: "EE".repeat(32).into(),
            holder_encryption_key: None,
            auditor_encrypted_amount: None,
            zk_proof: None,
        };

        let json = serde_json::to_string(&tx).unwrap();
        // Optional absent fields should not appear via skip_serializing_none.
        assert!(!json.contains("\"HolderEncryptionKey\""));
        assert!(!json.contains("\"ZKProof\""));
    }

    #[test]
    fn test_new_builder_and_accessors() {
        let mut tx = ConfidentialMPTConvert::new(
            "rUserAccount11111111111111111111".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "610F33".repeat(8).into(),
            "1000".into(),
            "AD3F".repeat(33).into(),
            "BC2E".repeat(33).into(),
            "EE".repeat(32).into(),
            None,
            None,
            None,
        )
        .with_fee(XRPAmount::from("20000"))
        .with_sequence(7);

        // with_fee/with_sequence route through the builder's
        // get_mut_common_fields() + into_self().
        assert_eq!(tx.get_common_fields().sequence, Some(7));
        assert_eq!(tx.get_common_fields().fee, Some(XRPAmount::from("20000")));
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::ConfidentialMPTConvert
        );
        // No currency amounts to validate, so Model::get_errors succeeds.
        assert!(tx.get_errors().is_ok());

        // Transaction::get_mut_common_fields (distinct from the builder's
        // same-named method) — disambiguate via UFCS.
        let common =
            <ConfidentialMPTConvert as Transaction<'_, NoFlags>>::get_mut_common_fields(&mut tx);
        assert_eq!(common.sequence, Some(7));
    }

    #[test]
    fn test_serialize_with_auditor_mirror() {
        let tx = ConfidentialMPTConvert {
            common_fields: CommonFields {
                account: "rUserAccount11111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTConvert,
                ..Default::default()
            },
            mptoken_issuance_id: "610F33".repeat(4).into(),
            mpt_amount: "750".into(),
            holder_encrypted_amount: "AD3F".repeat(33).into(),
            issuer_encrypted_amount: "BC2E".repeat(33).into(),
            blinding_factor: "EE".repeat(32).into(),
            holder_encryption_key: None,
            // Issuance with a registered AuditorEncryptionKey requires the mirror.
            auditor_encrypted_amount: Some("CD".repeat(66).into()),
            zk_proof: None,
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"AuditorEncryptedAmount\""));

        let round_tripped: ConfidentialMPTConvert = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }
}
