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

use super::confidential_mpt_constants::address_is_issuer;
use super::mptoken_issuance_set::validate_mptoken_issuance_id;
use super::{CommonFields, CommonTransactionBuilder};

/// A `ConfidentialMPTMergeInbox` transaction merges a holder's confidential
/// inbox balance (`CB_IN`) into their spending balance (`CB_S`) and resets
/// the inbox to a canonical encrypted-zero (XLS-0096 §9).
///
/// This transaction is **proof-free** — it carries no ZK proof, no
/// ciphertexts, and no commitments. The ledger performs the homomorphic
/// addition deterministically and bumps `ConfidentialBalanceVersion` to
/// invalidate any in-flight proofs that referenced the prior `CB_S`.
///
/// XLS-0096 §9 / §A.2.
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
pub struct ConfidentialMPTMergeInbox<'a> {
    /// The base fields for all transaction models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    /// 24-byte `MPTokenIssuanceID` (hex-encoded) identifying the issuance
    /// being merged. Same format as XLS-33's `MPTokenIssuanceID`.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,
}

impl<'a> Model for ConfidentialMPTMergeInbox<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        // The issuer has no personal confidential balance to merge, so it
        // cannot be the `Account` of a MergeInbox (`temMALFORMED`,
        // `ConfidentialMPTMergeInbox.cpp` preflight).
        if address_is_issuer(
            self.mptoken_issuance_id.as_ref(),
            self.common_fields.account.as_ref(),
        ) {
            return Err(XRPLModelException::ValueEqualsValue {
                field1: "account".into(),
                field2: "issuer".into(),
            });
        }
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTMergeInbox<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTMergeInbox<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTMergeInbox<'a> {
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
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTMergeInbox,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let tx = ConfidentialMPTMergeInbox {
            common_fields: CommonFields {
                account: "rUserAccount111111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTMergeInbox,
                fee: Some("12".into()),
                sequence: Some(42),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            mptoken_issuance_id: "610F33B8EBF7EC795F822A454FB852156AEFE50BE0CB8326338A81CD74801864"
                .into(),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTMergeInbox\""));
        assert!(json.contains("\"MPTokenIssuanceID\":\"610F33B8"));

        // Round-trip
        let round_tripped: ConfidentialMPTMergeInbox = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }

    #[test]
    fn test_new_builder_and_accessors() {
        let mut tx = ConfidentialMPTMergeInbox::new(
            "rUserAccount111111111111111111111".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "610F33".repeat(8).into(),
        )
        .with_fee(XRPAmount::from("20000"))
        .with_sequence(7);

        // with_fee/with_sequence route through the builder's
        // get_mut_common_fields() + into_self().
        assert_eq!(tx.get_common_fields().sequence, Some(7));
        assert_eq!(tx.get_common_fields().fee, Some(XRPAmount::from("20000")));
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::ConfidentialMPTMergeInbox
        );
        // No currency amounts to validate, so Model::get_errors succeeds.
        assert!(tx.get_errors().is_ok());

        // Transaction::get_mut_common_fields (distinct from the builder's
        // same-named method) — disambiguate via UFCS.
        let common =
            <ConfidentialMPTMergeInbox as Transaction<'_, NoFlags>>::get_mut_common_fields(&mut tx);
        assert_eq!(common.sequence, Some(7));
    }

    const ACCT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"; // B5F762..37E8
    // Issuance whose issuer AccountID (bytes 4..24) is ACCT.
    const ISS_OF_ACCT: &str = "00000001B5F762798A53D543A014CAF8B297CFF8F2F937E8";

    fn valid_merge_inbox() -> ConfidentialMPTMergeInbox<'static> {
        ConfidentialMPTMergeInbox {
            common_fields: CommonFields {
                account: ACCT.into(),
                transaction_type: TransactionType::ConfidentialMPTMergeInbox,
                ..Default::default()
            },
            // Arbitrary issuance whose issuer is not ACCT.
            mptoken_issuance_id: "610F33".repeat(8).into(),
        }
    }

    #[test]
    fn test_valid_merge_inbox_passes() {
        assert!(valid_merge_inbox().get_errors().is_ok());
    }

    #[test]
    fn test_account_is_issuer_rejected() {
        let mut tx = valid_merge_inbox();
        tx.mptoken_issuance_id = ISS_OF_ACCT.into();
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_issuance_id_rejected() {
        let mut tx = valid_merge_inbox();
        tx.mptoken_issuance_id = "610F33".into(); // too short
        assert!(tx.get_errors().is_err());
    }
}
