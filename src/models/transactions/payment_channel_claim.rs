use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    Model, ValidateCurrencies,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use crate::models::amount::XRPAmount;

use super::{CommonFields, CommonTransactionBuilder, FlagCollection};

/// Transactions of the PaymentChannelClaim type support additional values
/// in the Flags field. This enum represents those options.
///
/// See PaymentChannelClaim flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelclaim>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum PaymentChannelClaimFlag {
    /// Clear the channel's Expiration time. (Expiration is different from the
    /// channel's immutable CancelAfter time.) Only the source address of the
    /// payment channel can use this flag.
    TfRenew = 0x00010000,
    /// Request to close the channel. Only the channel source and destination
    /// addresses can use this flag. This flag closes the channel immediately if
    /// it has no more XRP allocated to it after processing the current claim,
    /// or if the destination address uses it. If the source address uses this
    /// flag when the channel still holds XRP, this schedules the channel to close
    /// after SettleDelay seconds have passed. (Specifically, this sets the Expiration
    /// of the channel to the close time of the previous ledger plus the channel's
    /// SettleDelay time, unless the channel already has an earlier Expiration time.)
    /// If the destination address uses this flag when the channel still holds XRP,
    /// any XRP that remains after processing the claim is returned to the source address.
    TfClose = 0x00020000,
}

/// Claim XRP from a payment channel, adjust
/// the payment channel's expiration, or both.
///
/// See PaymentChannelClaim:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelclaim>`
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
pub struct PaymentChannelClaim<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, PaymentChannelClaimFlag>,
    /// The unique ID of the channel, as a 64-character hexadecimal string.
    pub channel: Cow<'a, str>,
    /// Total amount of XRP, in drops, delivered by this channel after processing this claim.
    /// Required to deliver XRP. Must be more than the total amount delivered by the channel
    /// so far, but not greater than the Amount of the signed claim. Must be provided except
    /// when closing the channel.
    pub balance: Option<Cow<'a, str>>,
    /// The amount of XRP, in drops, authorized by the Signature. This must match the amount
    /// in the signed message. This is the cumulative amount of XRP that can be dispensed by
    /// the channel, including XRP previously redeemed.
    pub amount: Option<Cow<'a, str>>,
    /// The signature of this claim, as hexadecimal. The signed message contains the channel
    /// ID and the amount of the claim. Required unless the sender of the transaction is the
    /// source address of the channel.
    pub signature: Option<Cow<'a, str>>,
    /// The public key used for the signature, as hexadecimal. This must match the PublicKey
    /// stored in the ledger for the channel. Required unless the sender of the transaction
    /// is the source address of the channel and the Signature field is omitted. (The transaction
    /// includes the public key so that rippled can check the validity of the signature before
    /// trying to apply the transaction to the ledger.)
    pub public_key: Option<Cow<'a, str>>,
}

impl<'a> Model for PaymentChannelClaim<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, PaymentChannelClaimFlag> for PaymentChannelClaim<'a> {
    fn has_flag(&self, flag: &PaymentChannelClaimFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, PaymentChannelClaimFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, PaymentChannelClaimFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, PaymentChannelClaimFlag> for PaymentChannelClaim<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, PaymentChannelClaimFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> PaymentChannelClaim<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<PaymentChannelClaimFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        channel: Cow<'a, str>,
        amount: Option<Cow<'a, str>>,
        balance: Option<Cow<'a, str>>,
        public_key: Option<Cow<'a, str>>,
        signature: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::PaymentChannelClaim,
                account_txn_id,
                fee,
                Some(flags.unwrap_or_default()),
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
            channel,
            balance,
            amount,
            signature,
            public_key,
        }
    }

    /// Set balance
    pub fn with_balance(mut self, balance: Cow<'a, str>) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Set amount
    pub fn with_amount(mut self, amount: Cow<'a, str>) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set signature
    pub fn with_signature(mut self, signature: Cow<'a, str>) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Set public key
    pub fn with_public_key(mut self, public_key: Cow<'a, str>) -> Self {
        self.public_key = Some(public_key);
        self
    }

    /// Add flag
    pub fn with_flag(mut self, flag: PaymentChannelClaimFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    /// Set multiple flags
    pub fn with_flags(mut self, flags: Vec<PaymentChannelClaimFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            balance: Some("1000000".into()),
            amount: Some("1000000".into()),
            signature: Some("30440220718D264EF05CAED7C781FF6DE298DCAC68D002562C9BF3A07C1E721B420C0DAB02203A5A4779EF4D2CCC7BC3EF886676D803A9981B928D3B8ACA483B80ECA3CD7B9B".into()),
            public_key: Some("32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into()),
        };

        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"PaymentChannelClaim","Flags":0,"SigningPubKey":"","Channel":"C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198","Balance":"1000000","Amount":"1000000","Signature":"30440220718D264EF05CAED7C781FF6DE298DCAC68D002562C9BF3A07C1E721B420C0DAB02203A5A4779EF4D2CCC7BC3EF886676D803A9981B928D3B8ACA483B80ECA3CD7B9B","PublicKey":"32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A"}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: PaymentChannelClaim = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let payment_channel_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_balance("1000000".into())
        .with_amount("1000000".into())
        .with_signature("30440220718D264EF05CAED7C781FF6DE298DCAC68D002562C9BF3A07C1E721B420C0DAB02203A5A4779EF4D2CCC7BC3EF886676D803A9981B928D3B8ACA483B80ECA3CD7B9B".into())
        .with_public_key("32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into())
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("claiming from payment channel".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            payment_channel_claim.channel,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198"
        );
        assert_eq!(payment_channel_claim.balance.as_ref().unwrap(), "1000000");
        assert_eq!(payment_channel_claim.amount.as_ref().unwrap(), "1000000");
        assert!(payment_channel_claim.signature.is_some());
        assert!(payment_channel_claim.public_key.is_some());
        assert_eq!(
            payment_channel_claim.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(payment_channel_claim.common_fields.sequence, Some(123));
        assert_eq!(
            payment_channel_claim.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(payment_channel_claim.common_fields.source_tag, Some(12345));
        assert_eq!(
            payment_channel_claim
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_close_channel() {
        let close_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_flag(PaymentChannelClaimFlag::TfClose)
        .with_fee("12".into())
        .with_sequence(123);

        assert!(close_claim.has_flag(&PaymentChannelClaimFlag::TfClose));
        assert!(close_claim.balance.is_none());
        assert!(close_claim.amount.is_none());
        assert!(close_claim.signature.is_none());
        assert!(close_claim.public_key.is_none());
    }

    #[test]
    fn test_renew_channel() {
        let renew_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_flag(PaymentChannelClaimFlag::TfRenew)
        .with_fee("12".into())
        .with_sequence(123);

        assert!(renew_claim.has_flag(&PaymentChannelClaimFlag::TfRenew));
        assert_eq!(
            renew_claim.channel,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198"
        );
    }

    #[test]
    fn test_default() {
        let payment_channel_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        };

        assert_eq!(
            payment_channel_claim.common_fields.account,
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"
        );
        assert_eq!(
            payment_channel_claim.common_fields.transaction_type,
            TransactionType::PaymentChannelClaim
        );
        assert_eq!(
            payment_channel_claim.channel,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198"
        );
        assert!(payment_channel_claim.balance.is_none());
        assert!(payment_channel_claim.amount.is_none());
        assert!(payment_channel_claim.signature.is_none());
        assert!(payment_channel_claim.public_key.is_none());
    }

    #[test]
    fn test_multiple_flags() {
        let multi_flag_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_flags(vec![
            PaymentChannelClaimFlag::TfRenew,
            PaymentChannelClaimFlag::TfClose,
        ])
        .with_fee("12".into());

        assert!(multi_flag_claim.has_flag(&PaymentChannelClaimFlag::TfRenew));
        assert!(multi_flag_claim.has_flag(&PaymentChannelClaimFlag::TfClose));
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_claim = PaymentChannelClaim {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::PaymentChannelClaim,
                ..Default::default()
            },
            channel: "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            ..Default::default()
        }
        .with_ticket_sequence(456)
        .with_balance("500000".into())
        .with_amount("500000".into())
        .with_fee("12".into());

        assert_eq!(ticket_claim.common_fields.ticket_sequence, Some(456));
        assert_eq!(ticket_claim.balance.as_ref().unwrap(), "500000");
        assert_eq!(ticket_claim.amount.as_ref().unwrap(), "500000");
        // When using tickets, sequence should be None or 0
        assert!(ticket_claim.common_fields.sequence.is_none());
    }
}
