use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    amount::XRPAmount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::CommonFields;

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
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the PaymentChannelFund model.
    //
    // See PaymentChannelFund fields:
    // `<https://xrpl.org/paymentchannelfund.html#paymentchannelfund-fields>`
    /// Amount of XRP, in drops to add to the channel. Must be a positive amount of XRP.
    pub amount: XRPAmount<'a>,
    /// The unique ID of the channel to fund, as a 64-character hexadecimal string.
    pub channel: Cow<'a, str>,
    /// New Expiration time to set for the channel, in seconds since the Ripple Epoch.
    /// This must be later than either the current time plus the SettleDelay of the
    /// channel, or the existing Expiration of the channel. After the Expiration time,
    /// any transaction that would access the channel closes the channel without
    /// taking its normal action. Any unspent XRP is returned to the source address when
    /// the channel closes. (Expiration is separate from the channel's immutable
    /// CancelAfter time.) For more information, see the PayChannel ledger object type.
    pub expiration: Option<u32>,
}

impl<'a> Model for PaymentChannelFund<'a> {}

impl<'a> Transaction<'a, NoFlags> for PaymentChannelFund<'a> {
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

impl<'a> PaymentChannelFund<'a> {
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
        amount: XRPAmount<'a>,
        channel: Cow<'a, str>,
        expiration: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::PaymentChannelFund,
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
            amount,
            channel,
            expiration,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::amount::XRPAmount;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = PaymentChannelFund::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("200000"),
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            Some(543171558),
        );
        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"PaymentChannelFund","Flags":0,"SigningPubKey":"","Amount":"200000","Channel":"C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198","Expiration":543171558}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: PaymentChannelFund = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
