use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// This method retrieves all of sell offers for the specified NFToken.
///
/// See Nft Sell Offers:
/// `<https://xrpl.org/nft_sell_offers.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NftSellOffers<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The unique identifier of a NFToken object.
    pub nft_id: Cow<'a, str>,
}

impl<'a> Model for NftSellOffers<'a> {}

impl<'a> Request<'a> for NftSellOffers<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> NftSellOffers<'a> {
    pub fn new(id: Option<Cow<'a, str>>, nft_id: Cow<'a, str>) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::NFTSellOffers,
                id,
            },
            nft_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let req = NftSellOffers::new(
            Some("nso-1".into()),
            "00080000B4F4AFC5FBCBD76873F18006173D2193467D3EE70000099B00000000".into(),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: NftSellOffers = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"nft_sell_offers\""));
    }
}
