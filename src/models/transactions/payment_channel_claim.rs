use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};

use crate::models::amount::XRPAmount;

use super::{CommonFields, FlagCollection};

/// Transactions of the PaymentChannelClaim type support additional values
/// in the Flags field. This enum represents those options.
///
/// See PaymentChannelClaim flags:
/// `<https://xrpl.org/paymentchannelclaim.html#paymentchannelclaim-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
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
/// `<https://xrpl.org/paymentchannelclaim.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelClaim<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, PaymentChannelClaimFlag>,
    // The custom fields for the PaymentChannelClaim model.
    //
    // See PaymentChannelClaim fields:
    // `<https://xrpl.org/paymentchannelclaim.html#paymentchannelclaim-fields>`
    /// The unique ID of the channel, as a 64-character hexadecimal string.
    pub channel: Cow<'a, str>,
    /// otal amount of XRP, in drops, delivered by this channel after processing this claim.
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

impl<'a> Model for PaymentChannelClaim<'a> {}

impl<'a> Transaction<'a, PaymentChannelClaimFlag> for PaymentChannelClaim<'a> {
    fn has_flag(&self, flag: &PaymentChannelClaimFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, PaymentChannelClaimFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, PaymentChannelClaimFlag> {
        self.common_fields.get_mut_common_fields()
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
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        channel: Cow<'a, str>,
        amount: Option<Cow<'a, str>>,
        balance: Option<Cow<'a, str>>,
        public_key: Option<Cow<'a, str>>,
        signature: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::PaymentChannelClaim,
                account_txn_id,
                fee,
                flags: flags.unwrap_or_default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            channel,
            balance,
            amount,
            signature,
            public_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = PaymentChannelClaim::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198".into(),
            Some("1000000".into()),
            Some("1000000".into()),
            Some("32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into()),
            Some("30440220718D264EF05CAED7C781FF6DE298DCAC68D002562C9BF3A07C1E721B420C0DAB02203A5A4779EF4D2CCC7BC3EF886676D803A9981B928D3B8ACA483B80ECA3CD7B9B".into()),
        );
        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"PaymentChannelClaim","Flags":0,"Channel":"C1AE6DDDEEC05CF2978C0BAD6FE302948E9533691DC749DCDD3B9E5992CA6198","Balance":"1000000","Amount":"1000000","Signature":"30440220718D264EF05CAED7C781FF6DE298DCAC68D002562C9BF3A07C1E721B420C0DAB02203A5A4779EF4D2CCC7BC3EF886676D803A9981B928D3B8ACA483B80ECA3CD7B9B","PublicKey":"32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: PaymentChannelClaim = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
