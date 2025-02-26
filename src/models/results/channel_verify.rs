use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response format for the channel_verify method, which verifies the
/// signature of a payment channel claim.
///
/// See Channel Verify:
/// `<https://xrpl.org/channel_verify.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChannelVerify<'a> {
    /// The credentials specified in the request, if any.
    pub credentials: Option<Cow<'a, [Cow<'a, str>]>>,
    /// Whether the signature is valid for the stated amount, channel,
    /// and public key.
    pub signature_verified: bool,
    /// The identifying hash of the ledger that was used to generate
    /// this response.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version that was used to generate
    /// this response.
    pub ledger_index: Option<u32>,
    /// The ledger index of the current in-progress ledger version.
    pub ledger_current_index: Option<u32>,
    /// If true, the information comes from a validated ledger version.
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_verify_deserialize() {
        let json = r#"{
            "signature_verified": true,
            "status": "success"
        }"#;

        let result: ChannelVerify = serde_json::from_str(json).unwrap();

        assert!(result.signature_verified);
        assert!(result.credentials.is_none());
        assert!(result.ledger_hash.is_none());
        assert!(result.ledger_index.is_none());
        assert!(result.ledger_current_index.is_none());
        assert!(result.validated.is_none());
    }

    #[test]
    fn test_channel_verify_serialize() {
        let verify = ChannelVerify {
            signature_verified: true,
            credentials: None,
            ledger_hash: None,
            ledger_index: None,
            ledger_current_index: None,
            validated: None,
        };

        let json = serde_json::to_string(&verify).unwrap();
        let expected = r#"{"signature_verified":true}"#;

        assert_eq!(json, expected);
    }
}
