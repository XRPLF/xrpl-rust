pub mod issued_currency;
pub mod mpt_currency;
pub mod xrp;

use crate::models::Model;
use alloc::borrow::Cow;
pub use issued_currency::*;
pub use mpt_currency::*;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use strum_macros::Display;
pub use xrp::*;

use super::{IssuedCurrencyAmount, MPTAmount, XRPAmount};

const CURRENCY_OBJECT_FIELDS: &[&str] = &["mpt_issuance_id", "currency", "issuer"];
const MPT_CURRENCY_OBJECT_FIELDS: &[&str] = &["mpt_issuance_id"];
const ISSUED_CURRENCY_OBJECT_FIELDS: &[&str] = &["currency", "issuer"];
const XRP_CURRENCY_OBJECT_FIELDS: &[&str] = &["currency"];

fn reject_unknown_currency_fields<E>(
    obj: &serde_json::Map<alloc::string::String, serde_json::Value>,
    allowed: &'static [&'static str],
) -> Result<(), E>
where
    E: DeError,
{
    if let Some(field) = obj.keys().find(|field| !allowed.contains(&field.as_str())) {
        Err(E::unknown_field(field, allowed))
    } else {
        Ok(())
    }
}

pub trait ToAmount<'a, A> {
    fn to_amount(&self, value: Cow<'a, str>) -> A;
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Display)]
#[serde(untagged)]
pub enum Currency<'a> {
    /// MPTCurrency variant must be checked first: object with only `mpt_issuance_id`
    MPTCurrency(MPTCurrency<'a>),
    /// IssuedCurrency variant (requires both currency and issuer fields)
    IssuedCurrency(IssuedCurrency<'a>),
    /// XRP variant (only requires currency field set to "XRP")
    XRP(XRP<'a>),
}

impl<'de, 'a> Deserialize<'de> for Currency<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Some(obj) = value.as_object() {
            match (
                obj.contains_key("mpt_issuance_id"),
                obj.contains_key("currency"),
                obj.contains_key("issuer"),
            ) {
                // Pure MPT currency shape: accept only `{ mpt_issuance_id }`.
                // `currency`/`issuer` must be absent so hybrid objects cannot be
                // treated as MPT by silently discarding issued-currency keys.
                (true, false, false) => {
                    reject_unknown_currency_fields::<D::Error>(obj, MPT_CURRENCY_OBJECT_FIELDS)?;
                    serde_json::from_value::<MPTCurrency>(value)
                        .map(Currency::MPTCurrency)
                        .map_err(D::Error::custom)
                }

                // Issued-currency shape: accept only `{ currency, issuer }`.
                // If an MPT key is also present, this arm reaches the field allowlist
                // and rejects it as unknown instead of letting serde ignore it.
                (_, true, true) => {
                    reject_unknown_currency_fields::<D::Error>(obj, ISSUED_CURRENCY_OBJECT_FIELDS)?;
                    serde_json::from_value::<IssuedCurrency>(value)
                        .map(Currency::IssuedCurrency)
                        .map_err(D::Error::custom)
                }

                // XRP currency shape: `{ currency: "XRP" }` with no extra keys.
                // Other currency codes without an issuer are incomplete issued-currency
                // objects, so report the missing issuer rather than falling through.
                (_, true, false) => {
                    reject_unknown_currency_fields::<D::Error>(obj, XRP_CURRENCY_OBJECT_FIELDS)?;
                    let xrp = serde_json::from_value::<XRP>(value).map_err(D::Error::custom)?;
                    if xrp.currency == "XRP" {
                        Ok(Currency::XRP(xrp))
                    } else {
                        Err(D::Error::missing_field("issuer"))
                    }
                }

                // Partial issued-currency shapes get typed missing-field errors.
                // `currency` is the discriminator, so report it when callers supply only
                // `issuer` or mix `issuer` with an MPT key.
                (_, false, true) => Err(D::Error::missing_field("currency")),

                _ => {
                    if let Some(field) = obj.keys().next() {
                        Err(D::Error::unknown_field(field, CURRENCY_OBJECT_FIELDS))
                    } else {
                        Err(D::Error::invalid_length(
                            0,
                            &"a Currency object with mpt_issuance_id, currency+issuer, or currency='XRP'",
                        ))
                    }
                }
            }
        } else {
            Err(D::Error::custom("Currency must be a JSON object"))
        }
    }
}

impl<'a> Model for Currency<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        match self {
            Currency::MPTCurrency(mpt) => mpt.get_errors(),
            Currency::IssuedCurrency(issued_currency) => issued_currency.get_errors(),
            Currency::XRP(xrp) => xrp.get_errors(),
        }
    }
}

impl<'a> Default for Currency<'a> {
    fn default() -> Self {
        Self::XRP(XRP::new())
    }
}

impl<'a> From<MPTCurrency<'a>> for Currency<'a> {
    fn from(value: MPTCurrency<'a>) -> Self {
        Self::MPTCurrency(value)
    }
}

impl<'a> From<IssuedCurrency<'a>> for Currency<'a> {
    fn from(value: IssuedCurrency<'a>) -> Self {
        Self::IssuedCurrency(value)
    }
}

impl<'a> From<XRP<'a>> for Currency<'a> {
    fn from(value: XRP<'a>) -> Self {
        Self::XRP(value)
    }
}

impl<'a> From<IssuedCurrencyAmount<'a>> for Currency<'a> {
    fn from(value: IssuedCurrencyAmount<'a>) -> Self {
        IssuedCurrency::new(value.currency, value.issuer).into()
    }
}

impl<'a> From<XRPAmount<'a>> for Currency<'a> {
    fn from(_value: XRPAmount<'a>) -> Self {
        XRP::new().into()
    }
}

impl<'a> From<&MPTAmount<'a>> for Currency<'a> {
    fn from(value: &MPTAmount<'a>) -> Self {
        MPTCurrency::new(value.mpt_issuance_id.clone()).into()
    }
}

impl<'a> From<&IssuedCurrencyAmount<'a>> for Currency<'a> {
    fn from(value: &IssuedCurrencyAmount<'a>) -> Self {
        IssuedCurrency::new(value.currency.clone(), value.issuer.clone()).into()
    }
}

impl<'a> From<&XRPAmount<'a>> for Currency<'a> {
    fn from(_value: &XRPAmount<'a>) -> Self {
        XRP::new().into()
    }
}

#[cfg(test)]
mod tests_currency_enum {
    use alloc::string::ToString;

    use crate::models::Model;

    use super::*;
    use crate::models::transactions::test_fixtures::{ISSUER_ACCOUNT_ALT, MPT_ISSUANCE_ID};

    const VALID_ID: &str = MPT_ISSUANCE_ID;

    #[test]
    fn test_currency_deserialize_mpt() {
        let json = alloc::format!(r#"{{"mpt_issuance_id":"{VALID_ID}"}}"#);
        let cur: Currency = serde_json::from_str(&json).unwrap();
        assert!(matches!(cur, Currency::MPTCurrency(_)));
    }

    #[test]
    fn test_currency_mpt_json_round_trip() {
        let original = Currency::MPTCurrency(MPTCurrency::new(VALID_ID.into()));
        let json = serde_json::to_string(&original).unwrap();
        let decoded: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_currency_deserialize_issued_not_mpt() {
        let json = alloc::format!(r#"{{"currency":"USD","issuer":"{}"}}"#, ISSUER_ACCOUNT_ALT);
        let cur: Currency = serde_json::from_str(&json).unwrap();
        assert!(matches!(cur, Currency::IssuedCurrency(_)));
    }

    #[test]
    fn test_currency_mpt_get_errors_valid() {
        let cur = Currency::MPTCurrency(MPTCurrency::new(VALID_ID.into()));
        assert!(cur.get_errors().is_ok());
    }

    #[test]
    fn test_currency_mpt_get_errors_bad_id() {
        let cur = Currency::MPTCurrency(MPTCurrency::new("TOOSHORT".into()));
        assert!(cur.get_errors().is_err());
    }

    #[test]
    fn test_currency_mpt_hybrid_object_rejects_unknown_fields() {
        // Hybrid object with mpt_issuance_id AND issuer must not silently discard either shape.
        let json = alloc::format!(
            r#"{{"mpt_issuance_id":"{}","currency":"USD","issuer":"{}"}}"#,
            MPT_ISSUANCE_ID,
            ISSUER_ACCOUNT_ALT
        );
        let err = serde_json::from_str::<Currency>(&json).unwrap_err();
        assert!(err.to_string().contains("mpt_issuance_id"));
    }
}
