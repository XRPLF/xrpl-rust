use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    Amount, Currency, FlagCollection, IssuedCurrencyAmount, Model, XRPAmount, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Transactions of the AMMWithdraw type support additional values in the Flags field.
/// This enum represents those options.
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum AMMWithdrawFlag {
    TfLpToken = 0x00010000,
    TfWithdrawAll = 0x00020000,
    TfOneAssetWithdrawAll = 0x00040000,
    TfSingleAsset = 0x00080000,
    TfTwoAsset = 0x00100000,
    TfOneAssetLpToken = 0x00200000,
    TfLimitLpToken = 0x00400000,
}

/// Withdraw assets from an Automated Market Maker (AMM) instance by returning the
/// AMM's liquidity provider tokens (LP Tokens).
///
/// See AMMWithdraw transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammwithdraw>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AMMWithdraw<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, AMMWithdrawFlag>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// The amount of one asset to withdraw from the AMM.
    /// This must match the type of one of the assets (tokens or XRP) in the AMM's pool.
    pub amount: Option<Amount<'a>>,
    /// The amount of another asset to withdraw from the AMM.
    /// If present, this must match the type of the other asset in the AMM's pool
    /// and cannot be the same type as Amount.
    #[serde(rename = "Amount2")]
    pub amount2: Option<Amount<'a>>,
    /// The minimum effective price, in LP Token returned, to pay per unit of the asset
    /// to withdraw.
    pub e_price: Option<Amount<'a>>,
    /// How many of the AMM's LP Tokens to redeem.
    #[serde(rename = "LPTokenIn")]
    pub lp_token_in: Option<IssuedCurrencyAmount<'a>>,
}

impl Model for AMMWithdraw<'_> {
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
        } else {
            Ok(())
        }
    }
}

impl<'a> Transaction<'a, AMMWithdrawFlag> for AMMWithdraw<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, AMMWithdrawFlag> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AMMWithdrawFlag> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, AMMWithdrawFlag> for AMMWithdraw<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AMMWithdrawFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> AMMWithdraw<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<AMMWithdrawFlag>>,
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
        lp_token_in: Option<IssuedCurrencyAmount<'a>>,
    ) -> Self {
        AMMWithdraw {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMWithdraw,
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
            lp_token_in,
        }
    }

    pub fn with_amount(mut self, amount: Amount<'a>) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn with_amount2(mut self, amount2: Amount<'a>) -> Self {
        self.amount2 = Some(amount2);
        self
    }

    pub fn with_e_price(mut self, e_price: Amount<'a>) -> Self {
        self.e_price = Some(e_price);
        self
    }

    pub fn with_lp_token_in(mut self, lp_token_in: IssuedCurrencyAmount<'a>) -> Self {
        self.lp_token_in = Some(lp_token_in);
        self
    }

    pub fn with_flag(mut self, flag: AMMWithdrawFlag) -> Self {
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
        let default_txn = AMMWithdraw {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            amount: Some(Amount::XRPAmount(XRPAmount::from("1000000"))),
            lp_token_in: Some(IssuedCurrencyAmount::new(
                "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
                "100".into(),
            )),
            amount2: None,
            e_price: None,
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMWithdraw","Flags":0,"SigningPubKey":"","Asset":{"currency":"XRP"},"Asset2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"},"Amount":"1000000","LPTokenIn":{"currency":"039C99CD9AB0B70B32ECDA51EAAE471625608EA2","issuer":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","value":"100"}}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: AMMWithdraw = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let amm_withdraw = AMMWithdraw {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMWithdraw,
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
        .with_lp_token_in(IssuedCurrencyAmount::new(
            "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
            "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
            "100".into(),
        ))
        .with_flag(AMMWithdrawFlag::TfSingleAsset)
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert!(amm_withdraw.amount.is_some());
        assert!(amm_withdraw.lp_token_in.is_some());
        assert_eq!(amm_withdraw.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_withdraw.common_fields.sequence, Some(123));
        assert_eq!(
            amm_withdraw.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(amm_withdraw.common_fields.source_tag, Some(12345));
        assert_eq!(amm_withdraw.get_errors(), Ok(()));
    }

    #[test]
    fn test_amount2_requires_amount() {
        let withdraw = AMMWithdraw {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMWithdraw,
                fee: Some("10".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            amount2: Some(Amount::XRPAmount("10".into())),
            ..Default::default()
        };

        assert!(withdraw.get_errors().is_err());
    }

    #[test]
    fn test_e_price_requires_amount() {
        let withdraw = AMMWithdraw {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMWithdraw,
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

        assert!(withdraw.get_errors().is_err());
    }

    #[test]
    fn test_valid_configuration() {
        let withdraw = AMMWithdraw {
            common_fields: CommonFields {
                account: "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                transaction_type: TransactionType::AMMWithdraw,
                fee: Some("10".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
            )),
            amount: Some(Amount::XRPAmount("1000000".into())),
            amount2: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".into(),
                "100".into(),
            ))),
            ..Default::default()
        };

        assert!(withdraw.get_errors().is_ok());
    }
}
