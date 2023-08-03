use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    model::Model,
    transactions::{Flag, Memo, Signer, Transaction, TransactionType},
};

use crate::_serde::txn_flags;
use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};

/// Transactions of the TrustSet type support additional values
/// in the Flags field. This enum represents those options.
///
/// See TrustSet flags:
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum TrustSetFlag {
    /// Authorize the other party to hold currency issued by this account.
    /// (No effect unless using the asfRequireAuth AccountSet flag.) Cannot be unset.
    TfSetAuth = 0x00010000,
    /// Enable the No Ripple flag, which blocks rippling between two trust lines
    /// of the same currency if this flag is enabled on both.
    TfSetNoRipple = 0x00020000,
    /// Disable the No Ripple flag, allowing rippling on this trust line.)
    TfClearNoRipple = 0x00040000,
    /// Freeze the trust line.
    TfSetFreeze = 0x00100000,
    /// Unfreeze the trust line.
    TfClearFreeze = 0x00200000,
}

/// Create or modify a trust line linking two accounts.
///
/// See TrustSet:
/// `<https://xrpl.org/trustset.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TrustSet<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::trust_set")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: &'a str,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<&'a str>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<&'a str>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<&'a str>,
    /// Set of bit-flags for this transaction.
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<TrustSetFlag>>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo<'a>>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the TrustSet model.
    ///
    /// See TrustSet fields:
    /// `<https://xrpl.org/trustset.html#trustset-fields>`
    pub limit_amount: IssuedCurrencyAmount<'a>,
    pub quality_in: Option<u32>,
    pub quality_out: Option<u32>,
}

impl<'a> Default for TrustSet<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::TrustSet,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            limit_amount: Default::default(),
            quality_in: Default::default(),
            quality_out: Default::default(),
        }
    }
}

impl<'a> Model for TrustSet<'a> {}

impl<'a> Transaction for TrustSet<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        let mut flags = &Vec::new();

        if let Some(flag_set) = self.flags.as_ref() {
            flags = flag_set;
        }

        match flag {
            Flag::TrustSet(trust_set_flag) => match trust_set_flag {
                TrustSetFlag::TfClearFreeze => flags.contains(&TrustSetFlag::TfClearFreeze),
                TrustSetFlag::TfClearNoRipple => flags.contains(&TrustSetFlag::TfClearNoRipple),
                TrustSetFlag::TfSetAuth => flags.contains(&TrustSetFlag::TfSetAuth),
                TrustSetFlag::TfSetFreeze => flags.contains(&TrustSetFlag::TfSetFreeze),
                TrustSetFlag::TfSetNoRipple => flags.contains(&TrustSetFlag::TfSetNoRipple),
            },
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> TrustSet<'a> {
    pub fn new(
        account: &'a str,
        limit_amount: IssuedCurrencyAmount<'a>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        flags: Option<Vec<TrustSetFlag>>,
        memos: Option<Vec<Memo<'a>>>,
        signers: Option<Vec<Signer<'a>>>,
        quality_in: Option<u32>,
        quality_out: Option<u32>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::TrustSet,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags,
            memos,
            signers,
            limit_amount,
            quality_in,
            quality_out,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let default_txn = TrustSet::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            Some("12".into()),
            Some(12),
            Some(8007750),
            None,
            None,
            None,
            None,
            None,
            Some(vec![TrustSetFlag::TfClearNoRipple]),
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"TrustSet","Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","Fee":"12","Sequence":12,"LastLedgerSequence":8007750,"Flags":262144,"LimitAmount":{"currency":"USD","issuer":"rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc","value":"100"}}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = TrustSet::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            Some("12".into()),
            Some(12),
            Some(8007750),
            None,
            None,
            None,
            None,
            None,
            Some(vec![TrustSetFlag::TfClearNoRipple]),
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"TrustSet","Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","Fee":"12","Flags":262144,"LastLedgerSequence":8007750,"LimitAmount":{"currency":"USD","issuer":"rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc","value":"100"},"Sequence":12}"#;

        let txn_as_obj: TrustSet = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
