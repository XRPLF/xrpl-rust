use crate::models::transactions::mptoken_issuance_set::validate_mptoken_issuance_id;
use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};

/// An MPT (Multi-Purpose Token) amount.
///
/// MPT amounts represent a quantity of a specific Multi-Purpose Token,
/// identified by its issuance ID.
///
/// JSON shape per XRPL:
/// `{"value": "<u64 string>", "mpt_issuance_id": "<48-hex-char string>"}`
///
/// See MPToken:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptoken>`
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct MPTAmount<'a> {
    /// The token quantity, expressed as a non-negative integer string.
    pub value: Cow<'a, str>,
    /// The MPTokenIssuanceID that identifies which MPT this amount belongs to.
    /// Must be a 48-character ASCII hex string (24 bytes, Hash192).
    pub mpt_issuance_id: Cow<'a, str>,
}

pub(crate) fn validate_mpt_amount_value(field: &'static str, value: &str) -> XRPLModelResult<()> {
    // MPT amounts are unsigned integer strings in [0, i64::MAX] per XLS-33 / xrpld.
    // Match xrpl.js' /^[0-9]+$/ sanity check instead of Rust's looser u64 parser,
    // which would accept values such as "+1".
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: field.into(),
            format: "unsigned integer string".into(),
            found: value.to_string(),
        });
    }
    let n: u64 = value
        .parse()
        .map_err(|error| XRPLModelException::InvalidValueFormat {
            field: field.into(),
            format: alloc::format!("unsigned integer string ({error})"),
            found: value.to_string(),
        })?;
    if n > i64::MAX as u64 {
        return Err(XRPLModelException::InvalidValue {
            field: field.into(),
            expected: alloc::format!("MPT amount <= {} (i64::MAX)", i64::MAX),
            found: value.to_string(),
        });
    }
    Ok(())
}

impl<'a> Model for MPTAmount<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_mpt_amount_value("value", self.value.as_ref())?;
        validate_mptoken_issuance_id(self.mpt_issuance_id.as_ref())?;
        Ok(())
    }
}

impl<'a> MPTAmount<'a> {
    pub fn new(value: Cow<'a, str>, mpt_issuance_id: Cow<'a, str>) -> Self {
        Self {
            value,
            mpt_issuance_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Model;

    use super::*;
    use crate::models::transactions::test_fixtures::MPT_ISSUANCE_ID;

    const VALID_ID: &str = MPT_ISSUANCE_ID;

    #[test]
    fn test_mpt_amount_serde_roundtrip() {
        let amount = MPTAmount::new("100".into(), VALID_ID.into());
        let json = serde_json::to_string(&amount).unwrap();
        let decoded: MPTAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(amount, decoded);
    }

    #[test]
    fn test_mpt_amount_get_errors_valid() {
        let amount = MPTAmount::new("9223372036854775807".into(), VALID_ID.into());
        assert!(amount.get_errors().is_ok());
    }

    #[test]
    fn test_mpt_amount_get_errors_zero() {
        let amount = MPTAmount::new("0".into(), VALID_ID.into());
        assert!(amount.get_errors().is_ok());
    }

    #[test]
    fn test_mpt_amount_get_errors_bad_value_decimal() {
        let amount = MPTAmount::new("1.5".into(), VALID_ID.into());
        assert!(amount.get_errors().is_err());
    }

    #[test]
    fn test_mpt_amount_get_errors_bad_value_negative() {
        let amount = MPTAmount::new("-1".into(), VALID_ID.into());
        assert!(amount.get_errors().is_err());
    }

    #[test]
    fn test_mpt_amount_get_errors_bad_value_plus_prefix() {
        let amount = MPTAmount::new("+1".into(), VALID_ID.into());
        assert!(amount.get_errors().is_err());
    }

    #[test]
    fn test_mpt_amount_get_errors_rejects_above_i64_max() {
        // i64::MAX + 1 = 9223372036854775808: parses as u64 but exceeds protocol limit
        let amount = MPTAmount::new("9223372036854775808".into(), VALID_ID.into());
        assert!(amount.get_errors().is_err());
    }

    #[test]
    fn test_mpt_amount_get_errors_bad_id_too_short() {
        let amount = MPTAmount::new("100".into(), "DEAD".into());
        assert!(amount.get_errors().is_err());
    }

    #[test]
    fn test_mpt_amount_get_errors_bad_id_non_hex() {
        let bad_id = "Z".repeat(48);
        let amount = MPTAmount::new("100".into(), bad_id.as_str().into());
        assert!(amount.get_errors().is_err());
    }
}
