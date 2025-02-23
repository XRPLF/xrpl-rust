use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::amount::XRPAmount;

/// Response from an account_channels request, containing information about
/// an account's Payment Channels. This includes only channels where the
/// specified account is the channel's source, not the destination.
///
/// See Account Channels:
/// `<https://xrpl.org/account_channels.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountChannels<'a> {
    /// The address of the source/owner of the payment channels.
    pub account: Cow<'a, str>,
    /// Payment channels owned by this account.
    pub channels: Cow<'a, [Channel<'a>]>,
    /// The identifying Hash of the ledger version used to generate this
    /// response.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The Ledger Index of the ledger version used to generate this response.
    pub ledger_index: u32,
    /// If true, the information comes from a validated ledger version.
    pub validated: bool,
    /// The limit to how many channel objects were actually returned by this
    /// request.
    pub limit: Option<u32>,
    /// Server-defined value for pagination. Pass this to the next call to
    /// resume getting results where this call left off.
    pub marker: Option<Cow<'a, str>>,
}

/// Represents a single payment channel object in the XRP Ledger.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Channel<'a> {
    /// The owner of the channel, as an Address.
    pub account: Cow<'a, str>,
    /// The total amount of XRP, in drops allocated to this channel.
    pub amount: XRPAmount<'a>,
    /// The total amount of XRP, in drops, paid out from this channel.
    pub balance: XRPAmount<'a>,
    /// A unique ID for this channel, as a 64-character hexadecimal string.
    pub channel_id: Cow<'a, str>,
    /// The destination account of the channel, as an Address.
    pub destination_account: Cow<'a, str>,
    /// The public key for the payment channel in the XRP Ledger's
    /// base58 format.
    pub public_key: Option<Cow<'a, str>>,
    /// The public key for the payment channel in hexadecimal format.
    pub public_key_hex: Option<Cow<'a, str>>,
    /// The number of seconds the payment channel must stay open after the
    /// owner requests to close it.
    pub settle_delay: u64,
    /// Time, in seconds since the Ripple Epoch, when this channel is set
    /// to expire.
    pub expiration: Option<u64>,
    /// Time, in seconds since the Ripple Epoch, of this channel's immutable
    /// expiration.
    pub cancel_after: Option<u64>,
    /// A 32-bit unsigned integer to use as a source tag for payments through
    /// this channel.
    pub source_tag: Option<u32>,
    /// A 32-bit unsigned integer to use as a destination tag for payments
    /// through this channel.
    pub destination_tag: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_channels_deserialization() {
        let json = r#"{
            "account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
            "channels": [
                {
                    "account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                    "amount": "1000",
                    "balance": "0",
                    "channel_id": "C7F634794B79DB40E87179A9D1BF05D05797AE7E92DF8E93FD6656E8C4BE3AE7",
                    "destination_account": "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
                    "public_key": "aBR7mdD75Ycs8DRhMgQ4EMUEmBArF8SEh1hfjrT2V9DQTLNbJVqw",
                    "public_key_hex": "03CFD18E689434F032A4E84C63E2A3A6472D684EAF4FD52CA67742F3E24BAE81B2",
                    "settle_delay": 60
                }
            ],
            "ledger_hash": "27F530E5C93ED5C13994812787C1ED073C822BAEC7597964608F2C049C2ACD2D",
            "ledger_index": 71766343,
            "status": "success",
            "validated": true
        }"#;

        let account_channels: AccountChannels = serde_json::from_str(json).unwrap();

        assert_eq!(
            account_channels.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(account_channels.ledger_index, 71766343);
        assert_eq!(account_channels.validated, true);
        assert_eq!(
            account_channels.ledger_hash.as_ref().unwrap(),
            "27F530E5C93ED5C13994812787C1ED073C822BAEC7597964608F2C049C2ACD2D"
        );

        let channel = &account_channels.channels[0];
        assert_eq!(channel.account, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(channel.amount, "1000".into());
        assert_eq!(channel.balance, "0".into());
        assert_eq!(
            channel.channel_id,
            "C7F634794B79DB40E87179A9D1BF05D05797AE7E92DF8E93FD6656E8C4BE3AE7"
        );
        assert_eq!(
            channel.destination_account,
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"
        );
        assert_eq!(
            channel.public_key.as_ref().unwrap(),
            "aBR7mdD75Ycs8DRhMgQ4EMUEmBArF8SEh1hfjrT2V9DQTLNbJVqw"
        );
        assert_eq!(
            channel.public_key_hex.as_ref().unwrap(),
            "03CFD18E689434F032A4E84C63E2A3A6472D684EAF4FD52CA67742F3E24BAE81B2"
        );
        assert_eq!(channel.settle_delay, 60);
    }
}
