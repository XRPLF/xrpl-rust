use core::str::FromStr;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use bigdecimal::BigDecimal;

use crate::models::{transactions::offer_create::OfferCreateFlag, Amount, FlagCollection};

pub mod balance_parser;
pub mod nodes;
pub mod order_book_parser;
pub mod parser;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AccountBalance<'a> {
    pub account: Cow<'a, str>,
    pub balance: Balance<'a>,
}

#[derive(Debug, Clone)]
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

pub fn negate(value: &BigDecimal) -> BigDecimal {
    let zero = BigDecimal::from_str("0").unwrap();
    let working_value = zero - value;

    BigDecimal::from_str(&working_value.to_string()).unwrap()
}
