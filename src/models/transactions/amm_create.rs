use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, XRPAmount, XRPLModelResult};

use super::{
    exceptions::{XRPLAMMCreateException, XRPLTransactionException},
    CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType,
};

pub const AMM_CREATE_MAX_FEE: u16 = 1000;

/// Create a new Automated Market Maker (AMM) instance for trading a pair of
/// assets (fungible tokens or XRP).
///
/// Creates both an AMM object and a special AccountRoot object to represent the AMM.
/// Also transfers ownership of the starting balance of both assets from the sender to
/// the created AccountRoot and issues an initial balance of liquidity provider
/// tokens (LP Tokens) from the AMM account to the sender.
///
/// Caution: When you create the AMM, you should fund it with (approximately)
/// equal-value amounts of each asset.
/// Otherwise, other users can profit at your expense by trading with
/// this AMM (performing arbitrage).
/// The currency risk that liquidity providers take on increases with the
/// volatility (potential for imbalance) of the asset pair.
/// The higher the trading fee, the more it offsets this risk,
/// so it's best to set the trading fee based on the volatility of the asset pair.
///
/// See AMMCreate transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammcreate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AMMCreate<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The first of the two assets to fund this AMM with. This must be a positive amount.
    pub amount: Amount<'a>,
    /// The second of the two assets to fund this AMM with. This must be a positive amount.
    #[serde(rename = "Amount2")]
    pub amount2: Amount<'a>,
    /// The fee to charge for trades against this AMM instance, in units of 1/100,000;
    /// a value of 1 is equivalent to 0.001%.
    /// The maximum value is 1000, indicating a 1% fee.
    /// The minimum value is 0.
    pub trading_fee: u16,
}

impl Model for AMMCreate<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_trading_fee_error()?;
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMCreate<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for AMMCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Default for AMMCreate<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::AMMCreate,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: Amount::default(),
            amount2: Amount::default(),
            trading_fee: 0,
        }
    }
}

impl<'a> AMMCreate<'a> {
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
        amount: Amount<'a>,
        amount2: Amount<'a>,
        trading_fee: u16,
    ) -> AMMCreate<'a> {
        AMMCreate {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMCreate,
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
            amount2,
            trading_fee,
        }
    }

    // All common builder methods (with_fee, with_sequence, etc.) now come from the trait!
    // Only need transaction-specific methods here.

    fn get_trading_fee_error(&self) -> XRPLModelResult<()> {
        if self.trading_fee > AMM_CREATE_MAX_FEE {
            Err(
                XRPLTransactionException::from(XRPLAMMCreateException::TradingFeeOutOfRange {
                    max: AMM_CREATE_MAX_FEE,
                    found: self.trading_fee,
                })
                .into(),
            )
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IssuedCurrencyAmount;

    #[test]
    fn test_trading_fee_error() {
        let amm_create = AMMCreate {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMCreate,
                fee: Some(XRPAmount::from("1000")),
                last_ledger_sequence: Some(20),
                sequence: Some(1),
                ..Default::default()
            },
            amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            amount2: IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            trading_fee: 1001,
        };

        assert!(amm_create.get_errors().is_err());
    }

    #[test]
    fn test_no_error() {
        let amm_create = AMMCreate {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMCreate,
                fee: Some(XRPAmount::from("1000")),
                last_ledger_sequence: Some(20),
                sequence: Some(1),
                ..Default::default()
            },
            amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            amount2: IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "1000".into(),
            )
            .into(),
            trading_fee: 1000,
        };

        assert!(amm_create.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = AMMCreate {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMCreate,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
                "1000".into(),
            )),
            trading_fee: 500,
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMCreate","Flags":0,"SigningPubKey":"","Amount":"1000000","Amount2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"1000"},"TradingFee":500}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AMMCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern_with_trait() {
        let amm_create = AMMCreate {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
                "1000".into(),
            )),
            trading_fee: 500,
        }
        .with_fee("12".into()) // From CommonTransactionBuilder trait
        .with_sequence(123) // From CommonTransactionBuilder trait
        .with_last_ledger_sequence(7108682) // From CommonTransactionBuilder trait
        .with_source_tag(12345) // From CommonTransactionBuilder trait
        .with_memo(Memo::default()) // From CommonTransactionBuilder trait
        .with_account_txn_id("ABCD".into()) // From CommonTransactionBuilder trait
        .with_ticket_sequence(456); // From CommonTransactionBuilder trait

        assert_eq!(amm_create.trading_fee, 500);
        assert_eq!(amm_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_create.common_fields.sequence, Some(123));
        assert_eq!(amm_create.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(amm_create.common_fields.source_tag, Some(12345));
        assert_eq!(amm_create.common_fields.memos.as_ref().unwrap().len(), 1);
        assert_eq!(
            amm_create.common_fields.account_txn_id.as_ref().unwrap(),
            "ABCD"
        );
        assert_eq!(amm_create.common_fields.ticket_sequence, Some(456));
    }
}
