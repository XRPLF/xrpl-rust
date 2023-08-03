use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// You can protect your account by assigning a regular key pair to
/// it and using it instead of the master key pair to sign transactions
/// whenever possible. If your regular key pair is compromised, but
/// your master key pair is not, you can use a SetRegularKey transaction
/// to regain control of your account.
///
/// See SetRegularKey:
/// `<https://xrpl.org/setregularkey.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SetRegularKey<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::set_regular_key")]
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
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo<'a>>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the SetRegularKey model.
    ///
    /// See SetRegularKey fields:
    /// `<https://xrpl.org/setregularkey.html#setregularkey-fields>`
    pub regular_key: Option<&'a str>,
}

impl<'a> Default for SetRegularKey<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::SetRegularKey,
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
            regular_key: Default::default(),
        }
    }
}

impl<'a> Model for SetRegularKey<'a> {}

impl<'a> Transaction for SetRegularKey<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> SetRegularKey<'a> {
    pub fn new(
        account: &'a str,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        memos: Option<Vec<Memo<'a>>>,
        signers: Option<Vec<Signer<'a>>>,
        regular_key: Option<&'a str>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::SetRegularKey,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags: None,
            memos,
            signers,
            regular_key,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = SetRegularKey::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
            Some("12".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"),
        );
        let default_json = r#"{"TransactionType":"SetRegularKey","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Fee":"12","RegularKey":"rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = SetRegularKey::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
            Some("12".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"),
        );
        let default_json = r#"{"TransactionType":"SetRegularKey","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Fee":"12","RegularKey":"rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"}"#;

        let txn_as_obj: SetRegularKey = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
