use crate::models::amount::IssuedCurrencyAmount;
use crate::models::currency::ToAmount;
use crate::models::Model;
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct IssuedCurrency<'a> {
    pub currency: Cow<'a, str>,
    pub issuer: Cow<'a, str>,
}

impl<'a> Model for IssuedCurrency<'a> {}

impl<'a> ToAmount<'a, IssuedCurrencyAmount<'a>> for IssuedCurrency<'a> {
    fn to_amount(&self, value: Cow<'a, str>) -> IssuedCurrencyAmount<'a> {
        IssuedCurrencyAmount::new(self.currency.clone(), self.issuer.clone(), value)
    }
}

impl<'a> IssuedCurrency<'a> {
    pub fn new(currency: Cow<'a, str>, issuer: Cow<'a, str>) -> Self {
        Self { currency, issuer }
    }
}

impl<'a> From<IssuedCurrencyAmount<'a>> for IssuedCurrency<'a> {
    fn from(value: IssuedCurrencyAmount<'a>) -> Self {
        Self {
            currency: value.currency,
            issuer: value.issuer,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let issued_currency =
            IssuedCurrency::new("TST".into(), "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into());
        let issued_currency_json = serde_json::to_string(&issued_currency).unwrap();
        let actual = issued_currency_json.as_str();
        let expected = r#"{"currency":"TST","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"}"#;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_deserialize() {
        let issued_currency_json =
            r#"{"currency":"TST","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"}"#;
        let actual = serde_json::from_str(issued_currency_json).unwrap();
        let expected =
            IssuedCurrency::new("TST".into(), "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into());

        assert_eq!(expected, actual);
    }
}
