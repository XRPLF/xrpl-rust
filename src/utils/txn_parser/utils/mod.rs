use core::convert::TryFrom;
use core::str::FromStr;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use bigdecimal::BigDecimal;

use crate::{
    models::{
        transactions::{
            mptoken_issuance_set::validate_mptoken_issuance_id, offer_create::OfferCreateFlag,
        },
        Amount, FlagCollection, MPTAmount, Model, XRPLModelException,
    },
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
            Amount::MPTAmount(amount) => Self {
                // Use the MPTokenIssuanceID as the currency identifier for balance tracking.
                currency: amount.mpt_issuance_id,
                value: amount.value,
                issuer: None,
            },
        }
    }
}

impl<'a> TryFrom<Balance<'a>> for Amount<'a> {
    type Error = XRPLModelException;

    fn try_from(balance: Balance<'a>) -> Result<Self, Self::Error> {
        if balance.currency == "XRP" {
            Ok(Amount::XRPAmount(balance.value.into()))
        } else if let Some(issuer) = balance.issuer {
            Ok(Amount::IssuedCurrencyAmount(
                crate::models::IssuedCurrencyAmount {
                    currency: balance.currency,
                    value: balance.value,
                    issuer,
                },
            ))
        } else {
            validate_mptoken_issuance_id(balance.currency.as_ref())?;
            let amount = MPTAmount::new(balance.value, balance.currency);
            amount.get_errors()?;
            Ok(Amount::MPTAmount(amount))
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
    use crate::models::transactions::test_fixtures::{
        INVALID_MPT_ISSUANCE_ID_NON_HEX, INVALID_MPT_ISSUANCE_ID_SHORT, MPT_ISSUANCE_ID,
    };
    use crate::models::{IssuedCurrencyAmount, MPTAmount, XRPAmount};
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
        let amount = Amount::try_from(balance).unwrap();
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
        let amount = Amount::try_from(balance).unwrap();
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
    fn test_balance_from_mpt_amount() {
        let amount = Amount::MPTAmount(MPTAmount {
            value: "10".into(),
            mpt_issuance_id: MPT_ISSUANCE_ID.into(),
        });
        let balance: Balance = amount.into();
        assert_eq!(balance.currency, MPT_ISSUANCE_ID);
        assert_eq!(balance.value, "10");
        assert!(balance.issuer.is_none());
    }

    #[test]
    fn test_balance_to_mpt_amount_when_issuer_absent() {
        let balance = Balance {
            currency: MPT_ISSUANCE_ID.into(),
            value: "10".into(),
            issuer: None,
        };
        let amount = Amount::try_from(balance).unwrap();
        match amount {
            Amount::MPTAmount(mpt) => {
                assert_eq!(mpt.value, "10");
                assert_eq!(mpt.mpt_issuance_id, MPT_ISSUANCE_ID);
            }
            _ => panic!("expected MPTAmount"),
        }
    }

    #[test]
    fn test_balance_to_mpt_amount_rejects_short_issuance_id() {
        let balance = Balance {
            currency: INVALID_MPT_ISSUANCE_ID_SHORT.into(),
            value: "10".into(),
            issuer: None,
        };

        assert!(Amount::try_from(balance).is_err());
    }

    #[test]
    fn test_balance_to_mpt_amount_rejects_non_hex_issuance_id() {
        let balance = Balance {
            currency: INVALID_MPT_ISSUANCE_ID_NON_HEX.into(),
            value: "10".into(),
            issuer: None,
        };

        assert!(Amount::try_from(balance).is_err());
    }

    #[test]
    fn test_balance_mpt_round_trip() {
        let original = Amount::MPTAmount(MPTAmount {
            value: "10".into(),
            mpt_issuance_id: MPT_ISSUANCE_ID.into(),
        });
        let balance: Balance = original.clone().into();
        let round_tripped = Amount::try_from(balance).unwrap();
        assert_eq!(round_tripped, original);
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
