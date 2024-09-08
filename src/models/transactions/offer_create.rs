use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::Amount,
    Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use crate::models::amount::XRPAmount;

use super::{CommonFields, FlagCollection};

/// Transactions of the OfferCreate type support additional values
/// in the Flags field. This enum represents those options.
///
/// See OfferCreate flags:
/// `<https://xrpl.org/offercreate.html#offercreate-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
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
/// `<https://xrpl.org/offercreate.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OfferCreate<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, OfferCreateFlag>,
    // The custom fields for the OfferCreate model.
    //
    // See OfferCreate fields:
    // `<https://xrpl.org/offercreate.html#offercreate-fields>`
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

    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, OfferCreateFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, OfferCreateFlag> {
        self.common_fields.get_mut_common_fields()
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
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        taker_gets: Amount<'a>,
        taker_pays: Amount<'a>,
        expiration: Option<u32>,
        offer_sequence: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::OfferCreate,
                account_txn_id,
                fee,
                flags,
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
            taker_gets,
            taker_pays,
            expiration,
            offer_sequence,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};
    use alloc::vec;

    use super::*;

    #[test]
    fn test_has_flag() {
        let txn: OfferCreate = OfferCreate::new(
            "rpXhhWmCvDwkzNtRbm7mmD1vZqdfatQNEe".into(),
            None,
            Some("10".into()),
            Some(vec![OfferCreateFlag::TfImmediateOrCancel].into()),
            Some(72779837),
            None,
            Some(1),
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "0.3".into(),
            )),
            None,
            None,
        );
        assert!(txn.has_flag(&OfferCreateFlag::TfImmediateOrCancel));
        assert!(!txn.has_flag(&OfferCreateFlag::TfPassive));
    }

    #[test]
    fn test_get_transaction_type() {
        let txn: OfferCreate = OfferCreate::new(
            "rpXhhWmCvDwkzNtRbm7mmD1vZqdfatQNEe".into(),
            None,
            Some("10".into()),
            Some(vec![OfferCreateFlag::TfImmediateOrCancel].into()),
            Some(72779837),
            None,
            Some(1),
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "0.3".into(),
            )),
            None,
            None,
        );
        let actual = txn.get_transaction_type();
        let expect = TransactionType::OfferCreate;
        assert_eq!(actual, expect)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = OfferCreate::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            None,
            Some("12".into()),
            None,
            Some(7108682),
            None,
            Some(8),
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("6000000")),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "GKO".into(),
                "ruazs5h1qEsqpke88pcqnaseXdm6od2xc".into(),
                "2".into(),
            )),
            None,
            None,
        );
        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"OfferCreate","Fee":"12","LastLedgerSequence":7108682,"Sequence":8,"TakerGets":"6000000","TakerPays":{"currency":"GKO","issuer":"ruazs5h1qEsqpke88pcqnaseXdm6od2xc","value":"2"}}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: OfferCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
