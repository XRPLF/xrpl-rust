use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{FlagCollection, NoFlags};
use crate::models::{
    Model, ValidateCurrencies,
    amount::XRPAmount,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Add additional XRP to an open payment channel,
/// and optionally update the expiration time of the channel.
///
/// See PaymentChannelFund:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelfund>`
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
pub struct PaymentChannelFund<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
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

impl<'a> Model for PaymentChannelFund<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for PaymentChannelFund<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
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

    /// Set expiration
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::models::amount::XRPAmount;
    use crate::models::transactions::Memo;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = PaymentChannelFund {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelFund,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: XRPAmount::from("200000"),
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            expiration: Some(543171558),
        };

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

    #[test]
    fn test_builder_pattern() {
        let payment_channel_fund = PaymentChannelFund {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelFund,
                ..Default::default()
            },
            amount: XRPAmount::from("200000"),
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_expiration(543171558)
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("funding channel".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(payment_channel_fund.amount.0, "200000");
        assert_eq!(
            payment_channel_fund.channel,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198"
        );
        assert_eq!(payment_channel_fund.expiration, Some(543171558));
        assert_eq!(
            payment_channel_fund.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(payment_channel_fund.common_fields.sequence, Some(123));
        assert_eq!(
            payment_channel_fund.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(payment_channel_fund.common_fields.source_tag, Some(12345));
        assert_eq!(
            payment_channel_fund
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_default() {
        let payment_channel_fund = PaymentChannelFund {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelFund,
                ..Default::default()
            },
            amount: XRPAmount::from("200000"),
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        };

        assert_eq!(
            payment_channel_fund.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            payment_channel_fund.common_fields.transaction_type,
            TransactionType::PaymentChannelFund
        );
        assert_eq!(payment_channel_fund.amount.0, "200000");
        assert_eq!(
            payment_channel_fund.channel,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198"
        );
        assert!(payment_channel_fund.expiration.is_none());
    }

    #[test]
    fn test_without_expiration() {
        let payment_channel_fund = PaymentChannelFund {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelFund,
                fee: Some("12".into()),
                sequence: Some(123),
                ..Default::default()
            },
            amount: XRPAmount::from("500000"),
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            expiration: None, // Just funding without changing expiration
        };

        assert_eq!(payment_channel_fund.amount.0, "500000");
        assert!(payment_channel_fund.expiration.is_none());
        assert_eq!(
            payment_channel_fund.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(payment_channel_fund.common_fields.sequence, Some(123));
    }

    #[test]
    fn test_ticket_sequence() {
        let payment_channel_fund = PaymentChannelFund {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelFund,
                ..Default::default()
            },
            amount: XRPAmount::from("200000"),
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_ticket_sequence(456)
        .with_fee("12".into());

        assert_eq!(
            payment_channel_fund.common_fields.ticket_sequence,
            Some(456)
        );
        assert_eq!(
            payment_channel_fund.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        // When using tickets, sequence should be 0
        assert!(payment_channel_fund.common_fields.sequence.is_none());
    }
}
