use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This method retrieves all of the NFTs currently owned
/// by the specified account.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountNfts<'a> {
    /// The unique identifier of an account, typically the
    /// account's Address. The request returns a list of
    /// NFTs owned by this account.
    pub account: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// Limit the number of token pages to retrieve. Each page
    /// can contain up to 32 NFTs. The limit value cannot be
    /// lower than 20 or more than 400. The default is 100.
    pub limit: Option<u32>,
    /// Value from a previous paginated response. Resume
    /// retrieving data where that response left off.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::account_nfts")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountNfts<'a> {
    fn default() -> Self {
        AccountNfts {
            account: "",
            id: None,
            limit: None,
            marker: None,
            command: RequestMethod::AccountNfts,
        }
    }
}

impl<'a> Model for AccountNfts<'a> {}

impl<'a> AccountNfts<'a> {
    fn new(account: &'a str, id: Option<&'a str>, limit: Option<u32>, marker: Option<u32>) -> Self {
        Self {
            account,
            id,
            limit,
            marker,
            command: RequestMethod::AccountNfts,
        }
    }
}
