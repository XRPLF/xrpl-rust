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
        self.validate_currencies()
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
            mptoken_issuance_id: "610F33".repeat(4).into(),
            mpt_amount: "1000".into(),
            zk_proof: "a1".repeat(64).into(),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTClawback\""));
        assert!(json.contains("\"Holder\":\"rHolderAccount"));

        let round_tripped: ConfidentialMPTClawback = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }
}
