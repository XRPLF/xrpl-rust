use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// This method retrieves all of sell offers for the specified NFToken.
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
