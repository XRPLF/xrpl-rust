use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Marker, Request};

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
    pub marker: Option<Marker<'a>>,
}

impl<'a> Model for AccountNfts<'a> {}

impl<'a> Request<'a> for AccountNfts<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> AccountNfts<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        limit: Option<u32>,
        marker: Option<Marker<'a>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_account_nfts_serialization() {
        let req = AccountNfts::new(
            None,
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            Some(10),
            None,
        );
        let json = serde_json::to_string(&req).unwrap();
        println!("{}", json);
        assert!(json.contains("\"account\""));
        assert!(
            json.contains("\"command\":\"account_nfts\"")
                || json.contains("\"method\":\"account_nfts\"")
        );
    }
}
