use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response format for the channel_authorize method, which creates a
/// signature that can be used to redeem a specific amount of XRP from a
/// payment channel.
///
/// See Channel Authorize:
/// `<https://xrpl.org/channel_authorize.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChannelAuthorize<'a> {
    /// The signature for this claim, as a hexadecimal value. To process
    /// the claim, the destination account of the payment channel must send a
    /// `PaymentChannelClaim`` transaction with this signature, the exact
    /// Channel ID, XRP amount, and public key of the channel.
    pub signature: Cow<'a, str>,
    /// The credentials specified in the request, if any.
    pub credentials: Option<Cow<'a, [Cow<'a, str>]>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_authorize_deserialize() {
        let json = r#"{
            "signature": "304402204EF0AFB78AC23ED1C472E74F4299C0C21F1B21D07EFC0A3838A420F76D783A400220154FB11B6F54320666E4C36CA7F686C16A3A0456800BBC43746F34AF50290064"
        }"#;

        let result: ChannelAuthorize = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.signature,
            "304402204EF0AFB78AC23ED1C472E74F4299C0C21F1B21D07EFC0A3838A420F\
            76D783A400220154FB11B6F54320666E4C36CA7F686C16A3A0456800BBC43746\
            F34AF50290064"
        );
        assert_eq!(result.credentials, None);
    }

    #[test]
    fn test_channel_authorize_serialize() {
        let auth = ChannelAuthorize {
            signature: "304402204EF0AFB78AC23ED1C472E74F4299C0C21F1B21D07EF\
                        C0A3838A420F76D783A400220154FB11B6F54320666E4C36CA7\
                        F686C16A3A0456800BBC43746F34AF50290064"
                .into(),
            credentials: None,
        };

        let json = serde_json::to_string(&auth).unwrap();
        let expected = r#"{"signature":"304402204EF0AFB78AC23ED1C472E74F4299C0C21F1B21D07EFC0A3838A420F76D783A400220154FB11B6F54320666E4C36CA7F686C16A3A0456800BBC43746F34AF50290064"}"#;

        assert_eq!(json, expected);
    }
}
