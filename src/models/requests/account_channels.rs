use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This request returns information about an account's Payment
/// Channels. This includes only channels where the specified
/// account is the channel's source, not the destination.
/// (A channel's "source" and "owner" are the same.) All
/// information retrieved is relative to a particular version
/// of the ledger.
///
/// See Account Channels:
/// `<https://xrpl.org/account_channels.html>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::models::requests::AccountChannels;
///
/// let json = r#"{"account":"rH6ZiHU1PGamME2LvVTxrgvfjQpppWKGmr","marker":12345678,"command":"account_channels"}"#.to_string();
/// let model: AccountChannels = serde_json::from_str(&json).expect("");
/// let revert: Option<String> = match serde_json::to_string(&model) {
///     Ok(model) => Some(model),
///     Err(_) => None,
/// };
///
/// assert_eq!(revert, Some(json));
/// ```
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountChannels<'a> {
    /// The unique identifier of an account, typically the
    /// account's Address. The request returns channels where
    /// this account is the channel's owner/source.
    pub account: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// Limit the number of transactions to retrieve. Cannot
    /// be less than 10 or more than 400. The default is 200.
    pub limit: Option<u16>,
    /// The unique identifier of an account, typically the
    /// account's Address. If provided, filter results to
    /// payment channels whose destination is this account.
    pub destination_account: Option<&'a str>,
    /// Value from a previous paginated response.
    /// Resume retrieving data where that response left off.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::account_channels")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountChannels<'a> {
    fn default() -> Self {
        AccountChannels {
            account: "",
            id: None,
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            destination_account: None,
            marker: None,
            command: RequestMethod::AccountChannels,
        }
    }
}

impl<'a> Model for AccountChannels<'a> {}

impl<'a> AccountChannels<'a> {
    fn new(
        account: &'a str,
        id: Option<&'a str>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        limit: Option<u16>,
        destination_account: Option<&'a str>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            account,
            id,
            ledger_hash,
            ledger_index,
            limit,
            destination_account,
            marker,
            command: RequestMethod::AccountChannels,
        }
    }
}
