use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};

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
            mptoken_issuance_id:
                "610F33B8EBF7EC795F822A454FB852156AEFE50BE0CB8326338A81CD74801864"
                    .into(),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTMergeInbox\""));
        assert!(json.contains("\"MPTokenIssuanceID\":\"610F33B8"));

        // Round-trip
        let round_tripped: ConfidentialMPTMergeInbox = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }
}
