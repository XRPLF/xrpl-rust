use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    requests::RequestMethod,
    transactions::validate_credential_ids,
    Model, XRPLModelResult,
};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// The deposit_authorized command indicates whether one account
/// is authorized to send payments directly to another.
///
/// See Deposit Authorization:
/// `<https://xrpl.org/depositauth.html#deposit-authorization>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DepositAuthorized<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The recipient of a possible payment.
    pub destination_account: Cow<'a, str>,
    /// The sender of a possible payment.
    pub source_account: Cow<'a, str>,
    /// Credential IDs to consider when evaluating authorization.
    pub credentials: Option<Vec<Cow<'a, str>>>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
}

impl<'a> Model for DepositAuthorized<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_credential_ids(&self.credentials)
    }
}

impl<'a> Request<'a> for DepositAuthorized<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> DepositAuthorized<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        destination_account: Cow<'a, str>,
        source_account: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::DepositAuthorized,
                id,
            },
            source_account,
            destination_account,
            credentials: None,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
    }

    pub fn with_credentials(mut self, credentials: Vec<Cow<'a, str>>) -> Self {
        self.credentials = Some(credentials);
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::models::{Model, XRPLModelException};

    #[test]
    fn test_serde_round_trip() {
        let req = DepositAuthorized::new(
            Some("da-1".into()),
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            Some(LedgerIndex::Str("validated".into())),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: DepositAuthorized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"deposit_authorized\""));
        assert!(!serialized.contains("\"credentials\""));
    }

    #[test]
    fn test_with_credentials() {
        let req = DepositAuthorized::new(
            Some("da-1".into()),
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            Some(LedgerIndex::Str("validated".into())),
        )
        .with_credentials(vec![
            "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001".into(),
        ]);

        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("\"credentials\":[\"DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001\"]"));
    }

    #[test]
    fn test_credentials_empty_error() {
        let req = DepositAuthorized::new(
            None,
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            None,
        )
        .with_credentials(vec![]);

        assert_eq!(
            req.get_errors().unwrap_err(),
            XRPLModelException::ValueTooShort {
                field: "credential_ids".into(),
                min: 1,
                found: 0,
            }
        );
    }

    #[test]
    fn test_credentials_nine_entries_error() {
        let id = "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001";
        // 9 distinct IDs (vary last nibble to avoid duplicate rejection)
        let creds: Vec<Cow<'_, str>> = (0u8..9)
            .map(|i| format!("DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D{:03X}", i).into())
            .collect();
        let _ = id;
        let req = DepositAuthorized::new(
            None,
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            None,
        )
        .with_credentials(creds);

        assert_eq!(
            req.get_errors().unwrap_err(),
            XRPLModelException::ValueTooLong {
                field: "credential_ids".into(),
                max: 8,
                found: 9,
            }
        );
    }

    #[test]
    fn test_credentials_duplicate_error() {
        let id = "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001";
        let req = DepositAuthorized::new(
            None,
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            None,
        )
        .with_credentials(vec![id.into(), id.into()]);

        assert_eq!(
            req.get_errors().unwrap_err(),
            XRPLModelException::ValueEqualsValue {
                field1: "credential_ids".into(),
                field2: "credential_ids (duplicate entry)".into(),
            }
        );
    }

    #[test]
    fn test_credentials_case_variant_duplicate_error() {
        let req = DepositAuthorized::new(
            None,
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            None,
        )
        .with_credentials(vec![
            "dd40031c6c21164e7673a47c35513d52a6b0f1349a873ee0d188d8994cd4d001".into(),
            "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001".into(),
        ]);

        assert_eq!(
            req.get_errors().unwrap_err(),
            XRPLModelException::ValueEqualsValue {
                field1: "credential_ids".into(),
                field2: "credential_ids (duplicate entry)".into(),
            }
        );
    }

    #[test]
    fn test_credentials_none_ok() {
        let req = DepositAuthorized::new(
            None,
            "rDest11111111111111111111111111111".into(),
            "rSrc111111111111111111111111111111".into(),
            None,
            None,
        );
        assert!(req.get_errors().is_ok());
    }
}
