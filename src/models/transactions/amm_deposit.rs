use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::TransactionType, Amount, Currency, FlagCollection, IssuedCurrencyAmount, Model,
    XRPAmount, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction};

/// Transactions of the AMMDeposit type support additional values in the Flags field.
/// This enum represents those options.
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum AMMDepositFlag {
    TfLpToken = 0x00010000,
    TfSingleAsset = 0x00080000,
    TfTwoAsset = 0x00100000,
    TfOneAssetLpToken = 0x00200000,
    TfLimitLpToken = 0x00400000,
    TfTwoAssetIfEmpty = 0x00800000,
}

/// Deposit funds into an Automated Market Maker (AMM) instance
/// and receive the AMM's liquidity provider tokens (LP Tokens) in exchange.
///
/// You can deposit one or both of the assets in the AMM's pool.
/// If successful, this transaction creates a trust line to the AMM Account (limit 0)
/// to hold the LP Tokens.
///
/// See AMMDeposit transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammdeposit>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AMMDeposit<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, AMMDepositFlag>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// The amount of one asset to deposit to the AMM.
    /// If present, this must match the type of one of the assets (tokens or XRP)
    /// in the AMM's pool.
    pub amount: Option<Amount<'a>>,
    /// The amount of another asset to add to the AMM.
    /// If present, this must match the type of the other asset in the AMM's pool
    /// and cannot be the same asset as Amount.
    #[serde(rename = "Amount2")]
    pub amount2: Option<Amount<'a>>,
    /// The maximum effective price, in the deposit asset, to pay
    /// for each LP Token received.
    pub e_price: Option<Amount<'a>>,
    /// How many of the AMM's LP Tokens to buy.
    pub lp_token_out: Option<IssuedCurrencyAmount<'a>>,
}

impl Model for AMMDeposit<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        if self.amount2.is_some() && self.amount.is_none() {
            Err(XRPLModelException::FieldRequiresField {
                field1: "amount2".into(),
                field2: "amount".into(),
            })
        } else if self.e_price.is_some() && self.amount.is_none() {
            Err(XRPLModelException::FieldRequiresField {
                field1: "e_price".into(),
                field2: "amount".into(),
            })
        } else if self.lp_token_out.is_none() && self.amount.is_none() {
            Err(XRPLModelException::ExpectedOneOf(
                ["lp_token_out", "amount"].as_ref(),
            ))
        } else {
            Ok(())
        }
    }
}

impl<'a> Transaction<'a, AMMDepositFlag> for AMMDeposit<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, AMMDepositFlag> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AMMDepositFlag> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, AMMDepositFlag> for AMMDeposit<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AMMDepositFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Default for AMMDeposit<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::AMMDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::default(),
            asset2: Currency::default(),
            amount: None,
            amount2: None,
            e_price: None,
            lp_token_out: None,
        }
    }
}

impl<'a> AMMDeposit<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<AMMDepositFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        amount: Option<Amount<'a>>,
        amount2: Option<Amount<'a>>,
        e_price: Option<Amount<'a>>,
        lp_token_out: Option<IssuedCurrencyAmount<'a>>,
    ) -> AMMDeposit<'a> {
        AMMDeposit {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMDeposit,
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
            asset,
            asset2,
            amount,
            amount2,
            e_price,
            lp_token_out,
        }
    }

    /// Set the amount to deposit
    pub fn with_amount(mut self, amount: Amount<'a>) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the second amount to deposit
    pub fn with_amount2(mut self, amount2: Amount<'a>) -> Self {
        self.amount2 = Some(amount2);
        self
    }

    /// Set the effective price
    pub fn with_e_price(mut self, e_price: Amount<'a>) -> Self {
        self.e_price = Some(e_price);
        self
    }

    /// Set the LP token output amount
    pub fn with_lp_token_out(mut self, lp_token_out: IssuedCurrencyAmount<'a>) -> Self {
        self.lp_token_out = Some(lp_token_out);
        self
    }

    /// Add a flag
    pub fn with_flag(mut self, flag: AMMDepositFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{currency::XRP, IssuedCurrency};

    #[test]
    fn test_serde() {
        let default_txn = AMMDeposit {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            amount: Some(Amount::XRPAmount(XRPAmount::from("1000000"))),
            amount2: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
                "1000".into(),
            ))),
            e_price: None,
            lp_token_out: None,
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMDeposit","Flags":0,"SigningPubKey":"","Asset":{"currency":"XRP"},"Asset2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"},"Amount":"1000000","Amount2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"1000"}}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AMMDeposit = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let amm_deposit = AMMDeposit {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMDeposit,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            ..Default::default()
        }
        .with_amount(Amount::XRPAmount(XRPAmount::from("1000000")))
        .with_amount2(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            "1000".into(),
        )))
        .with_flag(AMMDepositFlag::TfTwoAsset)
        .with_fee("12".into()) // From CommonTransactionBuilder trait
        .with_sequence(123) // From CommonTransactionBuilder trait
        .with_last_ledger_sequence(7108682) // From CommonTransactionBuilder trait
        .with_source_tag(12345); // From CommonTransactionBuilder trait

        assert!(amm_deposit.amount.is_some());
        assert!(amm_deposit.amount2.is_some());
        assert_eq!(amm_deposit.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_deposit.common_fields.sequence, Some(123));
        assert_eq!(
            amm_deposit.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(amm_deposit.common_fields.source_tag, Some(12345));
        assert_eq!(amm_deposit.get_errors(), Ok(()));
    }

    #[test]
    fn test_no_amount() {
        let deposit = AMMDeposit {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMDeposit,
                fee: Some("10".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            e_price: Some(XRPAmount::from("10").into()),
            ..Default::default()
        };

        assert!(deposit.get_errors().is_err());
    }

    #[test]
    fn test_no_lp_token_out_or_amount() {
        let deposit = AMMDeposit {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMDeposit,
                fee: Some("10".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            ..Default::default()
        };

        assert!(deposit.get_errors().is_err());
    }

    #[test]
    fn test_amount2_no_amount() {
        let deposit = AMMDeposit {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMDeposit,
                fee: Some("10".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            amount2: Some(Amount::XRPAmount("10".into())),
            lp_token_out: Some(IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "10".into(),
            )),
            ..Default::default()
        };

        assert!(deposit.get_errors().is_err());
    }
}
