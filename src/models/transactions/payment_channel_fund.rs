use alloc::vec::Vec;
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    amount::XRPAmount,
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// Add additional XRP to an open payment channel,
/// and optionally update the expiration time of the channel.
///
/// See PaymentChannelFund:
/// `<https://xrpl.org/paymentchannelfund.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelFund<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::payment_channel_fund")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
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
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
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
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the PaymentChannelFund model.
    ///
    /// See PaymentChannelFund fields:
    /// `<https://xrpl.org/paymentchannelfund.html#paymentchannelfund-fields>`
    pub amount: XRPAmount<'a>,
    pub channel: Cow<'a, str>,
    pub expiration: Option<u32>,
}

impl<'a> Default for PaymentChannelFund<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::PaymentChannelFund,
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
            amount: Default::default(),
            channel: Default::default(),
            expiration: Default::default(),
        }
    }
}

impl<'a> Model for PaymentChannelFund<'a> {}

impl<'a> Transaction for PaymentChannelFund<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> PaymentChannelFund<'a> {
    pub fn new(
        account: Cow<'a, str>,
        channel: Cow<'a, str>,
        amount: XRPAmount<'a>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<Cow<'a, str>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        expiration: Option<u32>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::PaymentChannelFund,
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
            amount,
            channel,
            expiration,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::amount::XRPAmount;

    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = PaymentChannelFund::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            XRPAmount::from("200000"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(543171558),
        );
        let default_json = r#"{"TransactionType":"PaymentChannelFund","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Amount":"200000","Channel":"C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198","Expiration":543171558}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = PaymentChannelFund::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            XRPAmount::from("200000"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(543171558),
        );
        let default_json = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"PaymentChannelFund","Channel":"C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198","Amount":"200000","Expiration":543171558}"#;

        let txn_as_obj: PaymentChannelFund = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
