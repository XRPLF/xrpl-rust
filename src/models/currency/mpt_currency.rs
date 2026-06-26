use crate::models::transactions::mpt_common::validate_mptoken_issuance_id;
use crate::models::{Model, XRPLModelResult};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

/// An MPT (Multi-Purpose Token) currency identifier.
///
/// Identifies a specific MPT issuance as a currency specifier, used
/// in contexts where XRP or an issued currency could also appear.
///
/// JSON shape per XRPL (xrpl.js `MPTCurrency`):
/// `{"mpt_issuance_id": "<48-hex>"}`
///
/// See MPTokenIssuance:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>`
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct MPTCurrency<'a> {
    /// The MPTokenIssuanceID identifying this MPT. Must be a 48-character
    /// ASCII hex string (24 bytes, Hash192).
    pub mpt_issuance_id: Cow<'a, str>,
}

impl<'a> Model for MPTCurrency<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mpt_issuance_id.as_ref())?;
        Ok(())
    }
}

impl<'a> MPTCurrency<'a> {
    pub fn new(mpt_issuance_id: Cow<'a, str>) -> Self {
        Self { mpt_issuance_id }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Model;

    use super::*;

    const VALID_ID: &str = "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58";

    #[test]
    fn test_mpt_currency_serde_roundtrip() {
        let cur = MPTCurrency::new(VALID_ID.into());
        let json = serde_json::to_string(&cur).unwrap();
        let decoded: MPTCurrency = serde_json::from_str(&json).unwrap();
        assert_eq!(cur, decoded);
    }

    #[test]
    fn test_mpt_currency_get_errors_valid() {
        assert!(MPTCurrency::new(VALID_ID.into()).get_errors().is_ok());
    }

    #[test]
    fn test_mpt_currency_get_errors_bad_id_too_short() {
        assert!(MPTCurrency::new("XYZ".into()).get_errors().is_err());
    }

    #[test]
    fn test_mpt_currency_get_errors_bad_id_non_hex() {
        let bad_id = "Z".repeat(48);
        assert!(MPTCurrency::new(bad_id.as_str().into())
            .get_errors()
            .is_err());
    }
}
