use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelResult,
};

use super::{
    CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType,
    exceptions::{XRPLAMMCreateException, XRPLTransactionException},
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
pub struct AMMCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
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
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMCreate<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields /*  */
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
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
    use crate::models::{IssuedCurrencyAmount, XRPAmount};

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
    fn test_builder_pattern() {
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
        .with_memo(Memo {
            memo_data: Some("creating AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        }); // From CommonTransactionBuilder trait

        assert_eq!(amm_create.trading_fee, 500);
        assert_eq!(amm_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_create.common_fields.sequence, Some(123));
        assert_eq!(amm_create.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(amm_create.common_fields.source_tag, Some(12345));
        assert_eq!(amm_create.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let amm_create = AMMCreate {
            common_fields: CommonFields {
                account: "rAMMCreator123".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "EUR".into(),
                "rEURIssuer456".into(),
                "1000".into(),
            )),
            trading_fee: 250,
        };

        assert_eq!(amm_create.common_fields.account, "rAMMCreator123");
        assert_eq!(
            amm_create.common_fields.transaction_type,
            TransactionType::AMMCreate
        );
        assert_eq!(amm_create.trading_fee, 250);
        assert!(amm_create.common_fields.fee.is_none());
        assert!(amm_create.common_fields.sequence.is_none());
    }

    #[test]
    fn test_xrp_token_amm() {
        let xrp_token_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rXRPTokenAMM789".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("50000000")), // 50 XRP
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "BTC".into(),
                "rBTCIssuer123".into(),
                "0.5".into(), // 0.5 BTC
            )),
            trading_fee: 100, // 0.1% trading fee
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_memo(Memo {
            memo_data: Some("XRP-BTC AMM pool".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert!(matches!(xrp_token_amm.amount, Amount::XRPAmount(_)));
        assert!(matches!(
            xrp_token_amm.amount2,
            Amount::IssuedCurrencyAmount(_)
        ));
        assert_eq!(xrp_token_amm.trading_fee, 100);
        assert_eq!(xrp_token_amm.common_fields.sequence, Some(100));
        assert!(xrp_token_amm.validate().is_ok());
    }

    #[test]
    fn test_token_token_amm() {
        let token_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rTokenAMM111".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rUSDIssuer222".into(),
                "10000".into(), // 10,000 USD
            )),
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "EUR".into(),
                "rEURIssuer333".into(),
                "8500".into(), // 8,500 EUR (roughly equal value)
            )),
            trading_fee: 50, // 0.05% trading fee
        }
        .with_fee("15".into())
        .with_sequence(200);

        assert!(matches!(token_amm.amount, Amount::IssuedCurrencyAmount(_)));
        assert!(matches!(token_amm.amount2, Amount::IssuedCurrencyAmount(_)));
        assert_eq!(token_amm.trading_fee, 50);
        assert_eq!(token_amm.common_fields.sequence, Some(200));
        assert!(token_amm.validate().is_ok());
    }

    #[test]
    fn test_high_volatility_amm() {
        let volatile_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rVolatileAMM444".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "DOGE".into(),
                "rDOGEIssuer555".into(),
                "1000000".into(), // 1M DOGE
            )),
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "SHIB".into(),
                "rSHIBIssuer666".into(),
                "100000000".into(), // 100M SHIB
            )),
            trading_fee: 1000, // 1% trading fee for volatile assets
        }
        .with_fee("20".into())
        .with_sequence(300)
        .with_memo(Memo {
            memo_data: Some("high volatility meme coin AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(volatile_amm.trading_fee, 1000); // Maximum allowed fee
        assert_eq!(volatile_amm.common_fields.sequence, Some(300));
        assert!(volatile_amm.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rTicketAMM777".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("25000000")), // 25 XRP
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "ETH".into(),
                "rETHIssuer888".into(),
                "10".into(), // 10 ETH
            )),
            trading_fee: 30, // 0.03% trading fee
        }
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(ticket_amm.common_fields.ticket_sequence, Some(12345));
        // When using tickets, sequence should be None or 0
        assert!(ticket_amm.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rMultiMemoAMM999".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("100000000")), // 100 XRP
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USDC".into(),
                "rUSDCIssuer111".into(),
                "50".into(), // 50 USDC
            )),
            trading_fee: 25, // 0.025% trading fee
        }
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
        .with_sequence(400);

        assert_eq!(
            multi_memo_amm.common_fields.memos.as_ref().unwrap().len(),
            2
        );
        assert_eq!(multi_memo_amm.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_min_trading_fee() {
        let min_fee_amm = AMMCreate {
            common_fields: CommonFields {
                account: "rMinFeeAMM222".into(),
                transaction_type: TransactionType::AMMCreate,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("10000000")), // 10 XRP
            amount2: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USDT".into(),
                "rUSDTIssuer333".into(),
                "5".into(), // 5 USDT
            )),
            trading_fee: 0, // No trading fee
        }
        .with_fee("12".into())
        .with_sequence(500);

        assert_eq!(min_fee_amm.trading_fee, 0);
        assert!(min_fee_amm.validate().is_ok());
    }
}
