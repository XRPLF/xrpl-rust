use alloc::borrow::Cow;
use alloc::vec::Vec;
use bigdecimal::{BigDecimal, Zero};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, NoFlags, XRPLModelException, XRPLModelResult};
use crate::models::{
    Model, ValidateCurrencies,
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Accept offers to buy or sell an NFToken.
///
/// See NFTokenAcceptOffer:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokenacceptoffer>`
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
pub struct NFTokenAcceptOffer<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// Identifies the NFTokenOffer that offers to sell the NFToken.
    #[serde(rename = "NFTokenSellOffer")]
    pub nftoken_sell_offer: Option<Cow<'a, str>>,
    /// Identifies the NFTokenOffer that offers to buy the NFToken.
    #[serde(rename = "NFTokenBuyOffer")]
    pub nftoken_buy_offer: Option<Cow<'a, str>>,
    /// This field is only valid in brokered mode, and specifies the
    /// amount that the broker keeps as part of their fee for bringing
    /// the two offers together; the remaining amount is sent to the
    /// seller of the NFToken being bought. If specified, the fee must
    /// be such that, before applying the transfer fee, the amount that
    /// the seller would receive is at least as much as the amount
    /// indicated in the sell offer.
    #[serde(rename = "NFTokenBrokerFee")]
    pub nftoken_broker_fee: Option<Amount<'a>>,
}

impl<'a: 'static> Model for NFTokenAcceptOffer<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_brokered_mode_error()?;
        self._get_nftoken_broker_fee_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for NFTokenAcceptOffer<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for NFTokenAcceptOffer<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> NFTokenAcceptOfferError for NFTokenAcceptOffer<'a> {
    fn _get_brokered_mode_error(&self) -> XRPLModelResult<()> {
        if self.nftoken_broker_fee.is_some()
            && self.nftoken_sell_offer.is_none()
            && self.nftoken_buy_offer.is_none()
        {
            Err(XRPLModelException::ExpectedOneOf(&[
                "nftoken_sell_offer",
                "nftoken_buy_offer",
            ]))
        } else {
            Ok(())
        }
    }

    fn _get_nftoken_broker_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(nftoken_broker_fee) = &self.nftoken_broker_fee {
            let nftoken_broker_fee_decimal: BigDecimal = nftoken_broker_fee.clone().try_into()?;
            if nftoken_broker_fee_decimal.is_zero() {
                Err(XRPLModelException::ValueZero("nftoken_broker_fee".into()))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenAcceptOffer<'a> {
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
        nftoken_sell_offer: Option<Cow<'a, str>>,
        nftoken_buy_offer: Option<Cow<'a, str>>,
        nftoken_broker_fee: Option<Amount<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::NFTokenAcceptOffer,
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
            nftoken_sell_offer,
            nftoken_buy_offer,
            nftoken_broker_fee,
        }
    }

    /// Set sell offer
    pub fn with_nftoken_sell_offer(mut self, offer: Cow<'a, str>) -> Self {
        self.nftoken_sell_offer = Some(offer);
        self
    }

    /// Set buy offer
    pub fn with_nftoken_buy_offer(mut self, offer: Cow<'a, str>) -> Self {
        self.nftoken_buy_offer = Some(offer);
        self
    }

    /// Set broker fee
    pub fn with_nftoken_broker_fee(mut self, fee: Amount<'a>) -> Self {
        self.nftoken_broker_fee = Some(fee);
        self
    }
}

pub trait NFTokenAcceptOfferError {
    fn _get_brokered_mode_error(&self) -> XRPLModelResult<()>;
    fn _get_nftoken_broker_fee_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::models::{
        Model,
        amount::{Amount, IssuedCurrencyAmount, XRPAmount},
    };

    #[test]
    fn test_brokered_mode_error() {
        let nftoken_accept_offer = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            nftoken_broker_fee: Some(Amount::XRPAmount(XRPAmount::from("100"))),
            ..Default::default()
        };

        assert_eq!(
            nftoken_accept_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "Expected one of: nftoken_sell_offer, nftoken_buy_offer"
        );
    }

    #[test]
    fn test_broker_fee_error() {
        let nftoken_accept_offer = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            nftoken_sell_offer: Some(
                "68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77".into(),
            ),
            nftoken_broker_fee: Some(Amount::XRPAmount(XRPAmount::from("0"))),
            ..Default::default()
        };

        assert_eq!(
            nftoken_accept_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The value of the field `\"nftoken_broker_fee\"` is not allowed to be zero"
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "r9spUPhPBfB6kQeF6vPhwmtFwRhBh2JUCG".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                fee: Some("12".into()),
                last_ledger_sequence: Some(75447550),
                memos: Some(vec![Memo::new(
                    Some(
                        "61356534373538372D633134322D346663382D616466362D393666383562356435386437"
                            .to_string(),
                    ),
                    None,
                    None,
                )]),
                sequence: Some(68549302),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_sell_offer: Some(
                "68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77".into(),
            ),
            ..Default::default()
        };

        let default_json_str = r#"{"Account":"r9spUPhPBfB6kQeF6vPhwmtFwRhBh2JUCG","TransactionType":"NFTokenAcceptOffer","Fee":"12","Flags":0,"LastLedgerSequence":75447550,"Memos":[{"Memo":{"MemoData":"61356534373538372D633134322D346663382D616466362D393666383562356435386437","MemoFormat":null,"MemoType":null}}],"Sequence":68549302,"SigningPubKey":"","NFTokenSellOffer":"68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77"}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenAcceptOffer = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let accept_sell_offer = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rBuyerAccount123".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_sell_offer(
            "68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77".into(),
        )
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("accepting sell offer".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            accept_sell_offer.nftoken_sell_offer.as_ref().unwrap(),
            "68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77"
        );
        assert!(accept_sell_offer.nftoken_buy_offer.is_none());
        assert!(accept_sell_offer.nftoken_broker_fee.is_none());
        assert_eq!(
            accept_sell_offer.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(accept_sell_offer.common_fields.sequence, Some(123));
        assert_eq!(
            accept_sell_offer.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(accept_sell_offer.common_fields.source_tag, Some(12345));
        assert_eq!(
            accept_sell_offer
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_accept_buy_offer() {
        let accept_buy_offer = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rSellerAccount456".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_buy_offer(
            "1A2B3C4D5E6F7890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
        )
        .with_fee("15".into())
        .with_sequence(456);

        assert!(accept_buy_offer.nftoken_sell_offer.is_none());
        assert_eq!(
            accept_buy_offer.nftoken_buy_offer.as_ref().unwrap(),
            "1A2B3C4D5E6F7890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890"
        );
        assert!(accept_buy_offer.nftoken_broker_fee.is_none());
        assert_eq!(accept_buy_offer.common_fields.fee.as_ref().unwrap().0, "15");
        assert_eq!(accept_buy_offer.common_fields.sequence, Some(456));
        assert!(accept_buy_offer.validate().is_ok());
    }

    #[test]
    fn test_brokered_mode() {
        let brokered_accept = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rBrokerAccount789".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_sell_offer(
            "SELL1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
        )
        .with_nftoken_buy_offer(
            "BUY9876543210FEDCBA0987654321ABCDEF1234567890ABCDEF0987654321".into(),
        )
        .with_nftoken_broker_fee(Amount::XRPAmount(XRPAmount::from("50000"))) // 0.05 XRP broker fee
        .with_fee("20".into())
        .with_sequence(789)
        .with_memo(Memo {
            memo_data: Some("brokered transaction".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            brokered_accept.nftoken_sell_offer.as_ref().unwrap(),
            "SELL1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890"
        );
        assert_eq!(
            brokered_accept.nftoken_buy_offer.as_ref().unwrap(),
            "BUY9876543210FEDCBA0987654321ABCDEF1234567890ABCDEF0987654321"
        );
        assert!(brokered_accept.nftoken_broker_fee.is_some());
        assert_eq!(brokered_accept.common_fields.fee.as_ref().unwrap().0, "20");
        assert_eq!(brokered_accept.common_fields.sequence, Some(789));
        assert!(brokered_accept.validate().is_ok());
    }

    #[test]
    fn test_broker_fee_with_currency() {
        let currency_fee_accept = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rBrokerAccount999".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_sell_offer(
            "SELL5555555555ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
        )
        .with_nftoken_buy_offer(
            "BUY6666666666FEDCBA0987654321ABCDEF1234567890ABCDEF0987654321".into(),
        )
        .with_nftoken_broker_fee(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
            "5".into(),
        )))
        .with_fee("25".into())
        .with_sequence(111);

        assert!(currency_fee_accept.nftoken_broker_fee.is_some());
        assert!(
            !currency_fee_accept
                .nftoken_broker_fee
                .as_ref()
                .unwrap()
                .is_xrp()
        );
        assert_eq!(currency_fee_accept.common_fields.sequence, Some(111));
        assert!(currency_fee_accept.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_accept = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rTicketUser111".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_sell_offer(
            "TICKET1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
        )
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(ticket_accept.common_fields.ticket_sequence, Some(12345));
        assert_eq!(
            ticket_accept.nftoken_sell_offer.as_ref().unwrap(),
            "TICKET1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890"
        );
        // When using tickets, sequence should be None or 0
        assert!(ticket_accept.common_fields.sequence.is_none());
    }

    #[test]
    fn test_default() {
        let nftoken_accept_offer = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rTestAccount".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(nftoken_accept_offer.common_fields.account, "rTestAccount");
        assert_eq!(
            nftoken_accept_offer.common_fields.transaction_type,
            TransactionType::NFTokenAcceptOffer
        );
        assert!(nftoken_accept_offer.nftoken_sell_offer.is_none());
        assert!(nftoken_accept_offer.nftoken_buy_offer.is_none());
        assert!(nftoken_accept_offer.nftoken_broker_fee.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_accept = NFTokenAcceptOffer {
            common_fields: CommonFields {
                account: "rMultiMemoUser222".into(),
                transaction_type: TransactionType::NFTokenAcceptOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_nftoken_sell_offer(
            "MULTI1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
        )
        .with_memo(Memo {
            memo_data: Some("first memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("second memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(333);

        assert_eq!(
            multi_memo_accept
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_accept.common_fields.sequence, Some(333));
        assert!(multi_memo_accept.validate().is_ok());
    }
}
