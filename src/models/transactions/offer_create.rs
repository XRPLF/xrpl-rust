use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};

use crate::models::amount::XRPAmount;

use super::{CommonFields, CommonTransactionBuilder, FlagCollection};

/// Transactions of the OfferCreate type support additional values
/// in the Flags field. This enum represents those options.
///
/// See OfferCreate flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/offercreate>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum OfferCreateFlag {
    /// If enabled, the Offer does not consume Offers that exactly match it,
    /// and instead becomes an Offer object in the ledger.
    /// It still consumes Offers that cross it.
    TfPassive = 0x00010000,
    /// Treat the Offer as an Immediate or Cancel order. The Offer never creates
    /// an Offer object in the ledger: it only trades as much as it can by
    /// consuming existing Offers at the time the transaction is processed. If no
    /// Offers match, it executes "successfully" without trading anything.
    /// In this case, the transaction still uses the result code tesSUCCESS.
    TfImmediateOrCancel = 0x00020000,
    /// Treat the offer as a Fill or Kill order . The Offer never creates an Offer
    /// object in the ledger, and is canceled if it cannot be fully filled at the
    /// time of execution. By default, this means that the owner must receive the
    /// full TakerPays amount; if the tfSell flag is enabled, the owner must be
    /// able to spend the entire TakerGets amount instead.
    TfFillOrKill = 0x00040000,
    /// Exchange the entire TakerGets amount, even if it means obtaining more than
    /// the TakerPays amount in exchange.
    TfSell = 0x00080000,
}

/// Places an Offer in the decentralized exchange.
///
/// See OfferCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/offercreate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OfferCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, OfferCreateFlag>,
    /// The amount and type of currency being sold.
    pub taker_gets: Amount<'a>,
    /// The amount and type of currency being bought.
    pub taker_pays: Amount<'a>,
    /// Time after which the Offer is no longer active, in seconds since the Ripple Epoch.
    pub expiration: Option<u32>,
    /// An Offer to delete first, specified in the same way as OfferCancel.
    pub offer_sequence: Option<u32>,
}

impl<'a> Model for OfferCreate<'a> {}

impl<'a> Transaction<'a, OfferCreateFlag> for OfferCreate<'a> {
    fn has_flag(&self, flag: &OfferCreateFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, OfferCreateFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, OfferCreateFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, OfferCreateFlag> for OfferCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, OfferCreateFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> OfferCreate<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<OfferCreateFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        taker_gets: Amount<'a>,
        taker_pays: Amount<'a>,
        expiration: Option<u32>,
        offer_sequence: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::OfferCreate,
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
            taker_gets,
            taker_pays,
            expiration,
            offer_sequence,
        }
    }

    /// Set expiration
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Set offer sequence to cancel
    pub fn with_offer_sequence(mut self, offer_sequence: u32) -> Self {
        self.offer_sequence = Some(offer_sequence);
        self
    }

    /// Add flag
    pub fn with_flag(mut self, flag: OfferCreateFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    /// Set multiple flags
    pub fn with_flags(mut self, flags: Vec<OfferCreateFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};

    #[test]
    fn test_has_flag() {
        let txn = OfferCreate {
            common_fields: CommonFields {
                account: "rpXhhWmCvDwkzNtRbm7mmD1vZqdfatQNEe".into(),
                transaction_type: TransactionType::OfferCreate,
                fee: Some("10".into()),
                flags: vec![OfferCreateFlag::TfImmediateOrCancel].into(),
                last_ledger_sequence: Some(72779837),
                sequence: Some(1),
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("1000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "0.3".into(),
            )),
            ..Default::default()
        };

        assert!(txn.has_flag(&OfferCreateFlag::TfImmediateOrCancel));
        assert!(!txn.has_flag(&OfferCreateFlag::TfPassive));
    }

    #[test]
    fn test_get_transaction_type() {
        let txn = OfferCreate {
            common_fields: CommonFields {
                account: "rpXhhWmCvDwkzNtRbm7mmD1vZqdfatQNEe".into(),
                transaction_type: TransactionType::OfferCreate,
                fee: Some("10".into()),
                flags: vec![OfferCreateFlag::TfImmediateOrCancel].into(),
                last_ledger_sequence: Some(72779837),
                sequence: Some(1),
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("1000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "0.3".into(),
            )),
            ..Default::default()
        };

        let actual = txn.get_transaction_type();
        let expect = TransactionType::OfferCreate;
        assert_eq!(actual, &expect)
    }

    #[test]
    fn test_serde() {
        let default_txn = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                fee: Some("12".into()),
                last_ledger_sequence: Some(7108682),
                sequence: Some(8),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("6000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "GKO".into(),
                "ruazs5h1qEsqpke88pcqnaseXdm6od2xc".into(),
                "2".into(),
            )),
            ..Default::default()
        };

        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"OfferCreate","Fee":"12","Flags":0,"LastLedgerSequence":7108682,"Sequence":8,"SigningPubKey":"","TakerGets":"6000000","TakerPays":{"currency":"GKO","issuer":"ruazs5h1qEsqpke88pcqnaseXdm6od2xc","value":"2"}}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: OfferCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let offer_create = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("6000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "GKO".into(),
                "ruazs5h1qEsqpke88pcqnaseXdm6od2xc".into(),
                "2".into(),
            )),
            ..Default::default()
        }
        .with_expiration(1640995200) // Some future timestamp
        .with_offer_sequence(123) // Cancel previous offer
        .with_flag(OfferCreateFlag::TfPassive)
        .with_fee("12".into())
        .with_sequence(8)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("creating offer".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert!(offer_create.taker_gets.is_xrp());
        assert!(!offer_create.taker_pays.is_xrp());
        assert_eq!(offer_create.expiration, Some(1640995200));
        assert_eq!(offer_create.offer_sequence, Some(123));
        assert!(offer_create.has_flag(&OfferCreateFlag::TfPassive));
        assert_eq!(offer_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(offer_create.common_fields.sequence, Some(8));
        assert_eq!(
            offer_create.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(offer_create.common_fields.source_tag, Some(12345));
        assert_eq!(offer_create.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_immediate_or_cancel() {
        let ioc_offer = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("1000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "1".into(),
            )),
            ..Default::default()
        }
        .with_flag(OfferCreateFlag::TfImmediateOrCancel)
        .with_fee("12".into())
        .with_sequence(8);

        assert!(ioc_offer.has_flag(&OfferCreateFlag::TfImmediateOrCancel));
        assert!(!ioc_offer.has_flag(&OfferCreateFlag::TfPassive));
        assert!(ioc_offer.expiration.is_none()); // IOC doesn't need expiration
    }

    #[test]
    fn test_fill_or_kill() {
        let fok_offer = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "EUR".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "100".into(),
            )),
            taker_pays: Amount::XRPAmount(XRPAmount::from("50000000")),
            ..Default::default()
        }
        .with_flag(OfferCreateFlag::TfFillOrKill)
        .with_fee("12".into())
        .with_sequence(9);

        assert!(fok_offer.has_flag(&OfferCreateFlag::TfFillOrKill));
        assert!(!fok_offer.taker_gets.is_xrp());
        assert!(fok_offer.taker_pays.is_xrp());
    }

    #[test]
    fn test_sell_flag() {
        let sell_offer = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("1000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "1".into(),
            )),
            ..Default::default()
        }
        .with_flags(vec![OfferCreateFlag::TfSell, OfferCreateFlag::TfPassive])
        .with_fee("12".into())
        .with_sequence(10);

        assert!(sell_offer.has_flag(&OfferCreateFlag::TfSell));
        assert!(sell_offer.has_flag(&OfferCreateFlag::TfPassive));
        assert!(!sell_offer.has_flag(&OfferCreateFlag::TfImmediateOrCancel));
    }

    #[test]
    fn test_replace_offer() {
        let replace_offer = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("2000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "GKO".into(),
                "ruazs5h1qEsqpke88pcqnaseXdm6od2xc".into(),
                "3".into(),
            )),
            ..Default::default()
        }
        .with_offer_sequence(456) // Cancel offer with sequence 456
        .with_expiration(1672531200) // New expiration
        .with_fee("12".into())
        .with_sequence(11);

        assert_eq!(replace_offer.offer_sequence, Some(456));
        assert_eq!(replace_offer.expiration, Some(1672531200));
        assert_eq!(replace_offer.common_fields.sequence, Some(11));
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_offer = OfferCreate {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCreate,
                ..Default::default()
            },
            taker_gets: Amount::XRPAmount(XRPAmount::from("1000000")),
            taker_pays: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "1".into(),
            )),
            ..Default::default()
        }
        .with_ticket_sequence(789)
        .with_fee("12".into());

        assert_eq!(ticket_offer.common_fields.ticket_sequence, Some(789));
        // When using tickets, sequence should be None or 0
        assert!(ticket_offer.common_fields.sequence.is_none());
    }
}
