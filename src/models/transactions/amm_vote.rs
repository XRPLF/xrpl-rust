use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    Currency, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

pub const AMM_VOTE_MAX_TRADING_FEE: u16 = 1000;

/// Vote on the trading fee for an Automated Market Maker (AMM) instance.
///
/// Up to 8 accounts can vote in proportion to the amount of the AMM's LP Tokens
/// they hold.
/// Each new vote re-calculates the AMM's trading fee based on a weighted average
/// of the votes.
///
/// See AMMVote transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammvote>`
#[skip_serializing_none]
#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, Clone, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct AMMVote<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// The proposed fee to vote for, in units of 1/100,000; a value of 1 is equivalent
    /// to 0.001%.
    /// The maximum value is 1000, indicating a 1% fee.
    pub trading_fee: Option<u16>,
}

impl Model for AMMVote<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;
        if let Some(trading_fee) = self.trading_fee {
            if trading_fee > AMM_VOTE_MAX_TRADING_FEE {
                return Err(XRPLModelException::ValueTooHigh {
                    field: "trading_fee".into(),
                    max: AMM_VOTE_MAX_TRADING_FEE.into(),
                    found: trading_fee.into(),
                });
            }
        }

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMVote<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for AMMVote<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Default for AMMVote<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::AMMVote,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::default(),
            asset2: Currency::default(),
            trading_fee: None,
        }
    }
}

impl<'a> AMMVote<'a> {
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
        asset: Currency<'a>,
        asset2: Currency<'a>,
        trading_fee: Option<u16>,
    ) -> AMMVote<'a> {
        AMMVote {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMVote,
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
            asset,
            asset2,
            trading_fee,
        }
    }

    /// Set the trading fee to vote for
    pub fn with_trading_fee(mut self, trading_fee: u16) -> Self {
        self.trading_fee = Some(trading_fee);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{currency::XRP, IssuedCurrency};

    #[test]
    fn test_trading_fee_validation() {
        let amm_vote = AMMVote {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMVote,
                fee: Some(XRPAmount::from("1000")),
                sequence: Some(1),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            trading_fee: Some(1001), // Over the limit
        };

        assert!(amm_vote.get_errors().is_err());
    }

    #[test]
    fn test_valid_trading_fee() {
        let amm_vote = AMMVote {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMVote,
                fee: Some(XRPAmount::from("1000")),
                sequence: Some(1),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            trading_fee: Some(500), // Valid
        };

        assert!(amm_vote.get_errors().is_ok());
    }

    #[test]
    fn test_no_trading_fee() {
        let amm_vote = AMMVote {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMVote,
                fee: Some(XRPAmount::from("1000")),
                sequence: Some(1),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            trading_fee: None,
        };

        assert!(amm_vote.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = AMMVote {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMVote,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            trading_fee: Some(500),
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMVote","Flags":0,"SigningPubKey":"","Asset":{"currency":"XRP"},"Asset2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"},"TradingFee":500}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AMMVote = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let amm_vote = AMMVote {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMVote,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            ..Default::default()
        }
        .with_trading_fee(500)
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(amm_vote.trading_fee, Some(500));
        assert_eq!(amm_vote.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_vote.common_fields.sequence, Some(123));
        assert_eq!(amm_vote.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(amm_vote.common_fields.source_tag, Some(12345));
        assert!(amm_vote.get_errors().is_ok());
    }
}
