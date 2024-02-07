use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// This method retrieves all of the NFTs currently owned
/// by the specified account.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountNfts<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The unique identifier of an account, typically the
    /// account's Address. The request returns a list of
    /// NFTs owned by this account.
    pub account: Cow<'a, str>,
    /// Limit the number of token pages to retrieve. Each page
    /// can contain up to 32 NFTs. The limit value cannot be
    /// lower than 20 or more than 400. The default is 100.
    pub limit: Option<u32>,
    /// Value from a previous paginated response. Resume
    /// retrieving data where that response left off.
    pub marker: Option<u32>,
}

impl<'a> Model for AccountNfts<'a> {}

impl<'a> Request for AccountNfts<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> AccountNfts<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        limit: Option<u32>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::AccountNfts,
                id,
            },
            account,
            limit,
            marker,
        }
    }
}
