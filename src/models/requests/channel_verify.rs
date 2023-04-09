use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The channel_verify method checks the validity of a signature
/// that can be used to redeem a specific amount of XRP from a
/// payment channel.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChannelVerify<'a> {
    /// The Channel ID of the channel that provides the XRP.
    /// This is a 64-character hexadecimal string.
    pub channel_id: &'a str,
    /// The amount of XRP, in drops, the provided signature authorizes.
    pub amount: &'a str,
    /// The public key of the channel and the key pair that was used to
    /// create the signature, in hexadecimal or the XRP Ledger's
    /// base58 format.
    pub public_key: &'a str,
    /// The signature to verify, in hexadecimal.
    pub signature: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::channel_verify")]
    pub command: RequestMethod,
}

impl<'a> Default for ChannelVerify<'a> {
    fn default() -> Self {
        ChannelVerify {
            channel_id: "",
            amount: "",
            public_key: "",
            signature: "",
            id: None,
            command: RequestMethod::ChannelVerify,
        }
    }
}

impl<'a> Model for ChannelVerify<'a> {}

impl<'a> ChannelVerify<'a> {
    fn new(
        channel_id: &'a str,
        amount: &'a str,
        public_key: &'a str,
        signature: &'a str,
        id: Option<&'a str>,
    ) -> Self {
        Self {
            channel_id,
            amount,
            public_key,
            signature,
            id,
            command: RequestMethod::ChannelVerify,
        }
    }
}
