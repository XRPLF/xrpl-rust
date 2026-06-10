use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The server_state command asks the server for various
/// machine-readable information about the rippled server's
/// current state. The response is almost the same as the
/// server_info method, but uses units that are easier to
/// process instead of easier to read. (For example, XRP
/// values are given in integer drops instead of scientific
/// notation or decimal values, and time is given in
/// milliseconds instead of seconds.)
///
/// See Submit Multisigned:
/// `<https://xrpl.org/submit_multisigned.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SubmitMultisigned<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    pub tx_json: serde_json::Value,
    /// If true, and the transaction fails locally, do not
    /// retry or relay the transaction to other servers.
    pub fail_hard: Option<bool>,
}

impl<'a> Model for SubmitMultisigned<'a> {}

impl<'a> Request<'a> for SubmitMultisigned<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> SubmitMultisigned<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        tx_json: serde_json::Value,
        fail_hard: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::SubmitMultisigned,
                id,
            },
            fail_hard,
            tx_json,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let tx_json = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        });
        let req = SubmitMultisigned::new(Some("sm-1".into()), tx_json, Some(false));
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: SubmitMultisigned = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"submit_multisigned\""));
        assert!(serialized.contains("\"fail_hard\":false"));
    }
}
