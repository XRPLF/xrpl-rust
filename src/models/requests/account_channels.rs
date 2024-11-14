use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

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
/// use xrpl::models::requests::account_channels::AccountChannels;
///
/// let json = r#"{"command":"account_channels","account":"rH6ZiHU1PGamME2LvVTxrgvfjQpppWKGmr","marker":12345678}"#.to_string();
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
    /// Common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The unique identifier of an account, typically the
    /// account's Address. The request returns channels where
    /// this account is the channel's owner/source.
    pub account: Cow<'a, str>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// Limit the number of transactions to retrieve. Cannot
    /// be less than 10 or more than 400. The default is 200.
    pub limit: Option<u16>,
    /// The unique identifier of an account, typically the
    /// account's Address. If provided, filter results to
    /// payment channels whose destination is this account.
    pub destination_account: Option<Cow<'a, str>>,
    /// Value from a previous paginated response.
    /// Resume retrieving data where that response left off.
    pub marker: Option<u32>,
}

impl<'a> Model for AccountChannels<'a> {}

impl<'a> AccountChannels<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        destination_account: Option<Cow<'a, str>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        limit: Option<u16>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::AccountChannels,
                id,
            },
            account,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            limit,
            destination_account,
            marker,
        }
    }
}

impl<'a> Request<'a> for AccountChannels<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}
