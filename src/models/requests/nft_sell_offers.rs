use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This method retrieves all of sell offers for the specified NFToken.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NftSellOffers<'a> {
    /// The unique identifier of a NFToken object.
    pub nft_id: Cow<'a, str>,
    /// The request method.
    #[serde(default = "RequestMethod::nft_sell_offers")]
    pub command: RequestMethod,
}

impl<'a> Default for NftSellOffers<'a> {
    fn default() -> Self {
        NftSellOffers {
            nft_id: "".into(),
            command: RequestMethod::NftSellOffers,
        }
    }
}

impl<'a> Model for NftSellOffers<'a> {}

impl<'a> NftSellOffers<'a> {
    pub fn new(nft_id: Cow<'a, str>) -> Self {
        Self {
            nft_id,
            command: RequestMethod::NftSellOffers,
        }
    }
}
