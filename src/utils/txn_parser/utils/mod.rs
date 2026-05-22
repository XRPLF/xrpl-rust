use core::str::FromStr;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use bigdecimal::BigDecimal;

use crate::{
    models::{transactions::offer_create::OfferCreateFlag, Amount, FlagCollection},
    utils::exceptions::XRPLUtilsResult,
};

pub mod balance_parser;
pub mod nodes;
pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Balance<'a> {
    pub currency: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub issuer: Option<Cow<'a, str>>,
}

impl<'a: 'b, 'b> From<Amount<'a>> for Balance<'b> {
    fn from(amount: Amount<'a>) -> Self {
        match amount {
            Amount::XRPAmount(amount) => Self {
                currency: Cow::Borrowed("XRP"),
                value: amount.0,
                issuer: None,
            },
            Amount::IssuedCurrencyAmount(amount) => Self {
                currency: amount.currency,
                value: amount.value,
                issuer: Some(amount.issuer),
            },
        }
    }
}

impl<'a> From<Balance<'a>> for Amount<'a> {
    fn from(balance: Balance<'a>) -> Self {
        if balance.currency == "XRP" {
            Amount::XRPAmount(balance.value.into())
        } else {
            Amount::IssuedCurrencyAmount(crate::models::IssuedCurrencyAmount {
                currency: balance.currency,
                value: balance.value,
                issuer: balance.issuer.unwrap_or("".into()),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountBalance<'a> {
    pub account: Cow<'a, str>,
    pub balance: Balance<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountBalances<'a> {
    pub account: Cow<'a, str>,
    pub balances: Vec<Balance<'a>>,
}

#[derive(Debug, Clone)]
pub enum OfferStatus {
    Created,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct OfferChange<'a> {
    pub flags: FlagCollection<OfferCreateFlag>,
    pub taker_gets: Amount<'a>,
    pub taker_pays: Amount<'a>,
    pub sequence: u32,
    pub status: OfferStatus,
    pub maker_exchange_rate: Option<BigDecimal>,
    pub expiration_time: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AccountOfferChange<'a> {
    pub maker_account: Cow<'a, str>,
    pub offer_change: OfferChange<'a>,
}

#[derive(Debug, Clone)]
pub struct AccountOfferChanges<'a> {
    pub account: Cow<'a, str>,
    pub offer_changes: Vec<AccountOfferChange<'a>>,
}

#[derive(Debug, Clone)]
pub struct AccountObjectGroup<'a> {
    pub account: Cow<'a, str>,
    pub account_balances: Vec<AccountBalance<'a>>,
    pub account_offer_changes: Vec<AccountOfferChange<'a>>,
}

pub fn negate(value: &BigDecimal) -> XRPLUtilsResult<BigDecimal> {
    let zero = BigDecimal::from_str("0")?;
    let working_value = zero - value;

    Ok(BigDecimal::from_str(&working_value.to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IssuedCurrencyAmount, XRPAmount};
    use alloc::string::ToString;

    #[test]
    fn test_balance_from_xrp_amount() {
        let amount = Amount::XRPAmount(XRPAmount::from("100"));
        let balance: Balance = amount.into();
        assert_eq!(balance.currency, "XRP");
        assert_eq!(balance.value, "100");
        assert!(balance.issuer.is_none());
    }

    #[test]
    fn test_balance_from_issued_currency() {
        let amount = Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
            currency: "USD".into(),
            value: "5.5".into(),
            issuer: "rIssuer".into(),
        });
        let balance: Balance = amount.into();
        assert_eq!(balance.currency, "USD");
        assert_eq!(balance.value, "5.5");
        assert_eq!(balance.issuer.as_deref(), Some("rIssuer"));
    }

    #[test]
    fn test_balance_to_xrp_amount() {
        let balance = Balance {
            currency: "XRP".into(),
            value: "100".into(),
            issuer: None,
        };
        let amount: Amount = balance.into();
        match amount {
            Amount::XRPAmount(x) => assert_eq!(x.0, "100"),
            _ => panic!("expected XRPAmount"),
        }
    }

    #[test]
    fn test_balance_to_issued_currency() {
        let balance = Balance {
            currency: "USD".into(),
            value: "10".into(),
            issuer: Some("rIssuer".into()),
        };
        let amount: Amount = balance.into();
        match amount {
            Amount::IssuedCurrencyAmount(ic) => {
                assert_eq!(ic.currency, "USD");
                assert_eq!(ic.value, "10");
                assert_eq!(ic.issuer, "rIssuer");
            }
            _ => panic!("expected IssuedCurrencyAmount"),
        }
    }

    #[test]
    fn test_balance_to_issued_currency_no_issuer_falls_back_to_empty() {
        let balance = Balance {
            currency: "USD".into(),
            value: "10".into(),
            issuer: None,
        };
        let amount: Amount = balance.into();
        match amount {
            Amount::IssuedCurrencyAmount(ic) => assert_eq!(ic.issuer, ""),
            _ => panic!("expected IssuedCurrencyAmount"),
        }
    }

    #[test]
    fn test_negate_positive() {
        let v = BigDecimal::from_str("123.45").unwrap();
        let n = negate(&v).unwrap();
        assert_eq!(n.to_string(), "-123.45");
    }

    #[test]
    fn test_negate_negative() {
        let v = BigDecimal::from_str("-50").unwrap();
        let n = negate(&v).unwrap();
        assert_eq!(n.to_string(), "50");
    }

    #[test]
    fn test_negate_zero() {
        let v = BigDecimal::from_str("0").unwrap();
        let n = negate(&v).unwrap();
        assert_eq!(n.to_string(), "0");
    }
}
