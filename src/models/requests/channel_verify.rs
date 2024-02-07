use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{amount::XRPAmount, requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The channel_verify method checks the validity of a signature
/// that can be used to redeem a specific amount of XRP from a
/// payment channel.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChannelVerify<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The amount of XRP, in drops, the provided signature authorizes.
    pub amount: XRPAmount<'a>,
    /// The Channel ID of the channel that provides the XRP.
    /// This is a 64-character hexadecimal string.
    pub channel_id: Cow<'a, str>,
    /// The public key of the channel and the key pair that was used to
    /// create the signature, in hexadecimal or the XRP Ledger's
    /// base58 format.
    pub public_key: Cow<'a, str>,
    /// The signature to verify, in hexadecimal.
    pub signature: Cow<'a, str>,
}

impl<'a> Model for ChannelVerify<'a> {}

impl<'a> Request for ChannelVerify<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> ChannelVerify<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        amount: XRPAmount<'a>,
        channel_id: Cow<'a, str>,
        public_key: Cow<'a, str>,
        signature: Cow<'a, str>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::ChannelVerify,
                id,
            },
            channel_id,
            amount,
            public_key,
            signature,
        }
    }
}
