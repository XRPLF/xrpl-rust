use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// Create a unidirectional channel and fund it with XRP.
///
/// See PaymentChannelCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelcreate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// Amount of XRP, in drops, to deduct from the sender's balance and set aside in this channel.
    /// While the channel is open, the XRP can only go to the Destination address. When the channel
    /// closes, any unclaimed XRP is returned to the source address's balance.
    pub amount: XRPAmount<'a>,
    /// Address to receive XRP claims against this channel. This is also known as the
    /// "destination address" for the channel. Cannot be the same as the sender (Account).
    pub destination: Cow<'a, str>,
    /// Amount of time the source address must wait before closing the channel if it has unclaimed XRP.
    pub settle_delay: u32,
    /// The 33-byte public key of the key pair the source will use to sign claims against this channel,
    /// in hexadecimal. This can be any secp256k1 or Ed25519 public key.
    pub public_key: Cow<'a, str>,
    /// The time, in seconds since the Ripple Epoch, when this channel expires. Any transaction that
    /// would modify the channel after this time closes the channel without otherwise affecting it.
    /// This value is immutable; the channel can be closed earlier than this time but cannot remain
    /// open after this time.
    pub cancel_after: Option<u32>,
    /// Arbitrary tag to further specify the destination for this payment channel, such as a hosted
    /// recipient at the destination address.
    pub destination_tag: Option<u32>,
}

impl<'a> Model for PaymentChannelCreate<'a> {}

impl<'a> Transaction<'a, NoFlags> for PaymentChannelCreate<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for PaymentChannelCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> PaymentChannelCreate<'a> {
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
        destination: Cow<'a, str>,
        public_key: Cow<'a, str>,
        settle_delay: u32,
        cancel_after: Option<u32>,
        destination_tag: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::PaymentChannelCreate,
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
            destination,
            settle_delay,
            public_key,
            cancel_after,
            destination_tag,
        }
    }

    /// Set cancel after
    pub fn with_cancel_after(mut self, cancel_after: u32) -> Self {
        self.cancel_after = Some(cancel_after);
        self
    }

    /// Set destination tag
    pub fn with_destination_tag(mut self, destination_tag: u32) -> Self {
        self.destination_tag = Some(destination_tag);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                signing_pub_key: Some("".into()),
                source_tag: Some(11747),
                ..Default::default()
            },
            amount: XRPAmount::from("10000"),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 86400,
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            cancel_after: Some(533171558),
            destination_tag: Some(23480),
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"PaymentChannelCreate","Flags":0,"SigningPubKey":"","SourceTag":11747,"Amount":"10000","Destination":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","SettleDelay":86400,"PublicKey":"32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A","CancelAfter":533171558,"DestinationTag":23480}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: PaymentChannelCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let payment_channel_create = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("10000"),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 86400,
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            ..Default::default()
        }
        .with_cancel_after(533171558)
        .with_destination_tag(23480)
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(11747)
        .with_memo(Memo {
            memo_data: Some("creating payment channel".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(payment_channel_create.amount.0, "10000");
        assert_eq!(
            payment_channel_create.destination,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert_eq!(payment_channel_create.settle_delay, 86400);
        assert_eq!(
            payment_channel_create.public_key,
            "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A"
        );
        assert_eq!(payment_channel_create.cancel_after, Some(533171558));
        assert_eq!(payment_channel_create.destination_tag, Some(23480));
        assert_eq!(
            payment_channel_create.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(payment_channel_create.common_fields.sequence, Some(123));
        assert_eq!(
            payment_channel_create.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(payment_channel_create.common_fields.source_tag, Some(11747));
        assert_eq!(
            payment_channel_create
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
        let payment_channel_create = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("10000"),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 86400,
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            ..Default::default()
        };

        assert_eq!(
            payment_channel_create.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            payment_channel_create.common_fields.transaction_type,
            TransactionType::PaymentChannelCreate
        );
        assert_eq!(payment_channel_create.amount.0, "10000");
        assert_eq!(
            payment_channel_create.destination,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert_eq!(payment_channel_create.settle_delay, 86400);
        assert!(payment_channel_create.cancel_after.is_none());
        assert!(payment_channel_create.destination_tag.is_none());
    }

    #[test]
    fn test_without_optional_fields() {
        let payment_channel_create = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                fee: Some("12".into()),
                sequence: Some(123),
                ..Default::default()
            },
            amount: XRPAmount::from("5000"),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 3600, // 1 hour
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            cancel_after: None,
            destination_tag: None,
        };

        assert_eq!(payment_channel_create.amount.0, "5000");
        assert_eq!(payment_channel_create.settle_delay, 3600);
        assert!(payment_channel_create.cancel_after.is_none());
        assert!(payment_channel_create.destination_tag.is_none());
        assert_eq!(
            payment_channel_create.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(payment_channel_create.common_fields.sequence, Some(123));
    }

    #[test]
    fn test_ticket_sequence() {
        let payment_channel_create = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("10000"),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 86400,
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            ..Default::default()
        }
        .with_ticket_sequence(456)
        .with_fee("12".into());

        assert_eq!(
            payment_channel_create.common_fields.ticket_sequence,
            Some(456)
        );
        assert_eq!(
            payment_channel_create.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        // When using tickets, sequence should be None or 0
        assert!(payment_channel_create.common_fields.sequence.is_none());
    }

    #[test]
    fn test_long_lived_channel() {
        let payment_channel_create = PaymentChannelCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::PaymentChannelCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("100000000"), // 100 XRP
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            settle_delay: 604800, // 1 week
            public_key: "32D2471DB72B27E3310F355BB33E339BF26F8392D5A93D3BC0FC3B566612DA0F0A".into(),
            ..Default::default()
        }
        .with_cancel_after(1893456000) // Far future timestamp
        .with_destination_tag(98765)
        .with_fee("15".into());

        assert_eq!(payment_channel_create.amount.0, "100000000");
        assert_eq!(payment_channel_create.settle_delay, 604800);
        assert_eq!(payment_channel_create.cancel_after, Some(1893456000));
        assert_eq!(payment_channel_create.destination_tag, Some(98765));
    }
}
