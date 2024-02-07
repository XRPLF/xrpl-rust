use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::requests::XRPLChannelAuthorizeException;
use crate::{
    constants::CryptoAlgorithm,
    models::{requests::RequestMethod, Model},
    Err,
};

use super::{CommonFields, Request};

/// The channel_authorize method creates a signature that can  be
/// used to redeem a specific amount of XRP from a payment channel.
///
/// Warning: Do not send secret keys to untrusted servers or
/// through unsecured network connections. (This includes the
/// secret, seed, seed_hex, or passphrase fields of this request.)
/// You should only use this method on a secure, encrypted network
/// connection to a server you run or fully trust with your funds.
/// Otherwise, eavesdroppers could use your secret key to sign
/// claims and take all the money from this payment channel and
/// anything else using the same key pair.
///
/// See Set Up Secure Signing:
/// `<https://xrpl.org/set-up-secure-signing.html>`
///
/// See Channel Authorize:
/// `<https://xrpl.org/channel_authorize.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChannelAuthorize<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The unique ID of the payment channel to use.
    pub channel_id: Cow<'a, str>,
    /// Cumulative amount of XRP, in drops, to authorize.
    /// If the destination has already received a lesser amount
    /// of XRP from this channel, the signature created by this
    /// method can be redeemed for the difference.
    pub amount: Cow<'a, str>,
    /// The secret key to use to sign the claim. This must be
    /// the same key pair as the public key specified in the
    /// channel. Cannot be used with seed, seed_hex, or passphrase.
    pub secret: Option<Cow<'a, str>>,
    /// The secret seed to use to sign the claim. This must be
    /// the same key pair as the public key specified in the channel.
    /// Must be in the XRP Ledger's base58 format. If provided,
    /// you must also specify the key_type. Cannot be used with
    /// secret, seed_hex, or passphrase.
    pub seed: Option<Cow<'a, str>>,
    /// The secret seed to use to sign the claim. This must be the
    /// same key pair as the public key specified in the channel.
    /// Must be in hexadecimal format. If provided, you must also
    /// specify the key_type. Cannot be used with secret, seed,
    /// or passphrase.
    pub seed_hex: Option<Cow<'a, str>>,
    /// A string passphrase to use to sign the claim. This must be
    /// the same key pair as the public key specified in the channel.
    /// The key derived from this passphrase must match the public
    /// key specified in the channel. If provided, you must also
    /// specify the key_type. Cannot be used with secret, seed,
    /// or seed_hex.
    pub passphrase: Option<Cow<'a, str>>,
    /// The signing algorithm of the cryptographic key pair provided.
    /// Valid types are secp256k1 or ed25519. The default is secp256k1.
    pub key_type: Option<CryptoAlgorithm>,
}

impl<'a> Model for ChannelAuthorize<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_field_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => Ok(()),
        }
    }
}

impl<'a> Request for ChannelAuthorize<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> ChannelAuthorizeError for ChannelAuthorize<'a> {
    fn _get_field_error(&self) -> Result<(), XRPLChannelAuthorizeException> {
        let mut signing_methods = Vec::new();
        for method in [
            self.secret.clone(),
            self.seed.clone(),
            self.seed_hex.clone(),
            self.passphrase.clone(),
        ] {
            if method.is_some() {
                signing_methods.push(method)
            }
        }
        if signing_methods.len() != 1 {
            Err(XRPLChannelAuthorizeException::DefineExactlyOneOf {
                field1: "secret".into(),
                field2: "seed".into(),
                field3: "seed_hex".into(),
                field4: "passphrase".into(),
                resource: "".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> ChannelAuthorize<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        channel_id: Cow<'a, str>,
        amount: Cow<'a, str>,
        secret: Option<Cow<'a, str>>,
        seed: Option<Cow<'a, str>>,
        seed_hex: Option<Cow<'a, str>>,
        passphrase: Option<Cow<'a, str>>,
        key_type: Option<CryptoAlgorithm>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::ChannelAuthorize,
                id,
            },
            channel_id,
            amount,
            secret,
            seed,
            seed_hex,
            passphrase,
            key_type,
        }
    }
}

pub trait ChannelAuthorizeError {
    fn _get_field_error(&self) -> Result<(), XRPLChannelAuthorizeException>;
}

#[cfg(test)]
mod test_channel_authorize_errors {

    use crate::{constants::CryptoAlgorithm, models::Model};
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_fields_error() {
        let channel_authorize = ChannelAuthorize::new(
            None,
            "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".into(),
            "1000000".into(),
            None,
            Some("".into()),
            Some("".into()),
            None,
            Some(CryptoAlgorithm::SECP256K1),
        );

        assert_eq!(
            channel_authorize.validate().unwrap_err().to_string().as_str(),
            "The field `secret` can not be defined with `seed`, `seed_hex`, `passphrase`. Define exactly one of them. For more information see: "
        );
    }

    #[test]
    fn test_serde() {
        let req = ChannelAuthorize::new(
            None,
            "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".into(),
            "1000000".into(),
            None,
            Some("".into()),
            None,
            None,
            Some(CryptoAlgorithm::SECP256K1),
        );
        let serialized = serde_json::to_string(&req).unwrap();

        let deserialized: ChannelAuthorize = serde_json::from_str(&serialized).unwrap();

        assert_eq!(req, deserialized);
    }
}
