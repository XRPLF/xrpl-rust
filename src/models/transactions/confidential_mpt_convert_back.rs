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
    address_is_issuer, validate_hex_length, validate_mpt_amount, BLINDING_FACTOR_LENGTH,
    CIPHERTEXT_LENGTH, COMMITMENT_LENGTH, CONVERT_BACK_PROOF_LENGTH,
};
use super::mptoken_issuance_set::validate_mptoken_issuance_id;
use super::{CommonFields, CommonTransactionBuilder};

/// A `ConfidentialMPTConvertBack` transaction converts confidential MPT
/// value back to public form (XLS-0096 §10). The withdrawal amount is
/// revealed plaintext; the holder proves it doesn't exceed their balance
/// without revealing the balance itself.
///
/// The 816-byte `ZKProof` field carries:
///   - 128 B compact AND-composed sigma (balance ownership + key linkage)
///   - 688 B single Bulletproof (remainder is non-negative)
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
pub struct ConfidentialMPTConvertBack<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,

    /// Plaintext withdrawal amount (revealed publicly).
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext to be subtracted from holder's `CB_S`.
    pub holder_encrypted_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext to be subtracted from issuer mirror.
    pub issuer_encrypted_amount: Cow<'a, str>,

    /// 32-byte ElGamal randomness `r`. Revealed for deterministic
    /// verification of the ciphertexts above.
    pub blinding_factor: Cow<'a, str>,

    /// 33-byte Pedersen commitment to the holder's current balance.
    pub balance_commitment: Cow<'a, str>,

    /// 816-byte composite proof.
    #[serde(rename = "ZKProof")]
    pub zk_proof: Cow<'a, str>,

    /// 66-byte ciphertext for the auditor mirror. Required iff the
    /// issuance has an `AuditorEncryptionKey` registered.
    pub auditor_encrypted_amount: Option<Cow<'a, str>>,
}

impl<'a> Model for ConfidentialMPTConvertBack<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self._get_field_length_errors()?;
        self._get_issuer_role_error()?;
        self.validate_currencies()
    }
}

impl<'a> ConfidentialMPTConvertBack<'a> {
    /// The issuer holds value only through its mirror balance, so it cannot be
    /// the `Account` converting confidential value back to public
    /// (`temMALFORMED`, `ConfidentialMPTConvertBack.cpp` preflight).
    fn _get_issuer_role_error(&self) -> crate::models::XRPLModelResult<()> {
        if address_is_issuer(
            self.mptoken_issuance_id.as_ref(),
            self.common_fields.account.as_ref(),
        ) {
            return Err(XRPLModelException::ValueEqualsValue {
                field1: "account".into(),
                field2: "issuer".into(),
            });
        }
        Ok(())
    }

    fn _get_field_length_errors(&self) -> crate::models::XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        // A zero-amount ConvertBack is a no-op; rippled rejects it.
        validate_mpt_amount("mpt_amount", self.mpt_amount.as_ref(), true)?;
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
        validate_hex_length(
            "balance_commitment",
            self.balance_commitment.as_ref(),
            COMMITMENT_LENGTH,
        )?;
        validate_hex_length(
            "zk_proof",
            self.zk_proof.as_ref(),
            CONVERT_BACK_PROOF_LENGTH,
        )
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTConvertBack<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTConvertBack<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTConvertBack<'a> {
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
        balance_commitment: Cow<'a, str>,
        zk_proof: Cow<'a, str>,
        auditor_encrypted_amount: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTConvertBack,
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
            balance_commitment,
            zk_proof,
            auditor_encrypted_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let tx = ConfidentialMPTConvertBack {
            common_fields: CommonFields {
                account: "rUserAccount11111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTConvertBack,
                ..Default::default()
            },
            mptoken_issuance_id: "610F33".repeat(8).into(),
            mpt_amount: "500".into(),
            holder_encrypted_amount: "AD".repeat(66).into(),
            issuer_encrypted_amount: "BC".repeat(66).into(),
            blinding_factor: "12".repeat(32).into(),
            balance_commitment: "03".repeat(33).into(),
            zk_proof: "AB".repeat(816).into(),
            auditor_encrypted_amount: None,
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTConvertBack\""));
        assert!(json.contains("\"BalanceCommitment\""));

        let round_tripped: ConfidentialMPTConvertBack = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }

    #[test]
    fn test_new_builder_and_accessors() {
        let mut tx = ConfidentialMPTConvertBack::new(
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
            "500".into(),
            "AD".repeat(66).into(),
            "BC".repeat(66).into(),
            "12".repeat(32).into(),
            "03".repeat(33).into(),
            "AB".repeat(816).into(),
            None,
        )
        .with_fee(XRPAmount::from("15000"))
        .with_sequence(9);

        assert_eq!(tx.get_common_fields().sequence, Some(9));
        assert_eq!(tx.get_common_fields().fee, Some(XRPAmount::from("15000")));
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::ConfidentialMPTConvertBack
        );
        assert!(tx.get_errors().is_ok());

        let common =
            <ConfidentialMPTConvertBack as Transaction<'_, NoFlags>>::get_mut_common_fields(
                &mut tx,
            );
        assert_eq!(common.sequence, Some(9));
    }

    // ACCT's AccountID is B5F762..37E8.
    const ACCT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    // Issuance whose issuer AccountID (bytes 4..24) is ACCT.
    const ISS_OF_ACCT: &str = "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8";

    fn valid_convert_back() -> ConfidentialMPTConvertBack<'static> {
        ConfidentialMPTConvertBack {
            common_fields: CommonFields {
                account: ACCT.into(),
                transaction_type: TransactionType::ConfidentialMPTConvertBack,
                ..Default::default()
            },
            // Arbitrary issuance whose issuer is not ACCT.
            mptoken_issuance_id: "610F33".repeat(8).into(),
            mpt_amount: "500".into(),
            holder_encrypted_amount: "AD".repeat(66).into(),
            issuer_encrypted_amount: "BC".repeat(66).into(),
            blinding_factor: "12".repeat(32).into(),
            balance_commitment: "03".repeat(33).into(),
            zk_proof: "AB".repeat(816).into(),
            auditor_encrypted_amount: None,
        }
    }

    #[test]
    fn test_valid_convert_back_passes() {
        assert!(valid_convert_back().get_errors().is_ok());
    }

    #[test]
    fn test_zero_amount_convert_back_rejected() {
        // Unlike Convert, a zero-amount ConvertBack is a no-op and rejected.
        let mut tx = valid_convert_back();
        tx.mpt_amount = "0".into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_account_is_issuer_rejected() {
        let mut tx = valid_convert_back();
        tx.mptoken_issuance_id = ISS_OF_ACCT.into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_amount_above_mpt_max_rejected() {
        let mut tx = valid_convert_back();
        tx.mpt_amount = "9223372036854775808".into();
        assert!(tx.get_errors().is_err());
    }
}
