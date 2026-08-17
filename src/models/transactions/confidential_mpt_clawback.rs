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
    address_is_issuer, validate_hex_length, validate_mpt_amount, CLAWBACK_PROOF_LENGTH,
};
use super::mptoken_issuance_set::{validate_holder_address, validate_mptoken_issuance_id};
use super::{CommonFields, CommonTransactionBuilder};

/// A `ConfidentialMPTClawback` transaction is an issuer-only operation
/// that reclaims a holder's confidential balance, decrypting it via the
/// issuer's mirror key and burning the result (XLS-0096 §11).
///
/// The 64-byte `ZKProof` is a compact sigma proof that the holder's
/// `IssuerEncryptedBalance` ciphertext encrypts the plaintext `MPTAmount`
/// the issuer is reclaiming. The transaction simultaneously decreases both
/// `OutstandingAmount` and `ConfidentialOutstandingAmount` — effectively
/// burning the clawed-back tokens.
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
pub struct ConfidentialMPTClawback<'a> {
    /// `Account` here is the issuer initiating the clawback.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    /// The holder being clawed back.
    pub holder: Cow<'a, str>,

    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,

    /// The plaintext total amount being reclaimed (decrypted by the issuer
    /// from the holder's `IssuerEncryptedBalance` mirror).
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,

    /// 64-byte compact Clawback sigma proof.
    #[serde(rename = "ZKProof")]
    pub zk_proof: Cow<'a, str>,
}

impl<'a> Model for ConfidentialMPTClawback<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self._get_holder_error()?;
        self._get_field_length_errors()?;
        self._get_issuer_role_error()?;
        self.validate_currencies()
    }
}

impl<'a> ConfidentialMPTClawback<'a> {
    /// An issuer cannot claw back from itself (`temMALFORMED`).
    fn _get_holder_error(&self) -> crate::models::XRPLModelResult<()> {
        validate_holder_address(self.holder.as_ref())?;
        if self.holder == self.common_fields.account {
            return Err(XRPLModelException::ValueEqualsValue {
                field1: "holder".into(),
                field2: "account".into(),
            });
        }
        Ok(())
    }

    /// Clawback is issuer-only: `Account` MUST be the issuance's issuer,
    /// otherwise rippled rejects with `temMALFORMED`
    /// (`ConfidentialMPTClawback.cpp` preflight `account != issuer`). Requires a
    /// well-formed `MPTokenIssuanceID`, so it is checked after the length pass.
    fn _get_issuer_role_error(&self) -> crate::models::XRPLModelResult<()> {
        if !address_is_issuer(
            self.mptoken_issuance_id.as_ref(),
            self.common_fields.account.as_ref(),
        ) {
            return Err(XRPLModelException::InvalidValue {
                field: "account".into(),
                expected: "the issuance's issuer (ConfidentialMPTClawback is issuer-only)".into(),
                found: self.common_fields.account.as_ref().into(),
            });
        }
        Ok(())
    }

    fn _get_field_length_errors(&self) -> crate::models::XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        validate_mpt_amount("mpt_amount", self.mpt_amount.as_ref(), true)?;
        validate_hex_length("zk_proof", self.zk_proof.as_ref(), CLAWBACK_PROOF_LENGTH)
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTClawback<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTClawback<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTClawback<'a> {
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
        holder: Cow<'a, str>,
        mptoken_issuance_id: Cow<'a, str>,
        mpt_amount: Cow<'a, str>,
        zk_proof: Cow<'a, str>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTClawback,
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
            holder,
            mptoken_issuance_id,
            mpt_amount,
            zk_proof,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let tx = ConfidentialMPTClawback {
            common_fields: CommonFields {
                account: "rIssuerAccount11111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTClawback,
                ..Default::default()
            },
            holder: "rHolderAccount11111111111111".into(),
            mptoken_issuance_id: "610F33".repeat(8).into(),
            mpt_amount: "1000".into(),
            zk_proof: "a1".repeat(64).into(),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTClawback\""));
        assert!(json.contains("\"Holder\":\"rHolderAccount"));

        let round_tripped: ConfidentialMPTClawback = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }

    #[test]
    fn test_new_builder_and_accessors() {
        let mut tx = ConfidentialMPTClawback::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            // Clawback is issuer-only: the issuance ID must embed the submitting
            // account (rHb9...) as its issuer — sequence(8 hex) || issuerAccountID.
            "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8".into(),
            "1000".into(),
            "a1".repeat(64).into(),
        )
        .with_fee(XRPAmount::from("15000"))
        .with_sequence(9);

        assert_eq!(tx.get_common_fields().sequence, Some(9));
        assert_eq!(tx.get_common_fields().fee, Some(XRPAmount::from("15000")));
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::ConfidentialMPTClawback
        );
        assert!(tx.get_errors().is_ok());

        let common =
            <ConfidentialMPTClawback as Transaction<'_, NoFlags>>::get_mut_common_fields(&mut tx);
        assert_eq!(common.sequence, Some(9));
    }

    const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"; // B5F762..37E8
    const HOLDER: &str = "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK"; // D528B6..705F
    // Issuance whose issuer AccountID (bytes 4..24) is ISSUER.
    const ISS_OF_ISSUER: &str = "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8";

    fn valid_clawback() -> ConfidentialMPTClawback<'static> {
        ConfidentialMPTClawback {
            common_fields: CommonFields {
                account: ISSUER.into(),
                transaction_type: TransactionType::ConfidentialMPTClawback,
                ..Default::default()
            },
            holder: HOLDER.into(),
            mptoken_issuance_id: ISS_OF_ISSUER.into(),
            mpt_amount: "1000".into(),
            zk_proof: "a1".repeat(64).into(),
        }
    }

    #[test]
    fn test_valid_clawback_passes() {
        assert!(valid_clawback().get_errors().is_ok());
    }

    #[test]
    fn test_non_issuer_account_rejected() {
        // Clawback is issuer-only: an issuance whose issuer is not Account fails.
        let mut tx = valid_clawback();
        tx.mptoken_issuance_id = "610F33".repeat(8).into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_self_clawback_rejected() {
        // holder == account (which is also the issuer here) is still malformed.
        let mut tx = valid_clawback();
        tx.holder = ISSUER.into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_zero_amount_rejected() {
        let mut tx = valid_clawback();
        tx.mpt_amount = "0".into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_amount_above_mpt_max_rejected() {
        let mut tx = valid_clawback();
        tx.mpt_amount = "9223372036854775808".into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_malformed_holder_rejected() {
        let mut tx = valid_clawback();
        tx.holder = "not_a_classic_address".into();
        assert!(tx.get_errors().is_err());
    }
}
