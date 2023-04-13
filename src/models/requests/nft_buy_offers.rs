use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This method retrieves all of buy offers for the specified NFToken.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NftBuyOffers<'a> {
    /// The unique identifier of a NFToken object.
    pub nft_id: &'a str,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// Limit the number of NFT buy offers to retrieve.
    /// This value cannot be lower than 50 or more than 500.
    /// The default is 250.
    pub limit: Option<u16>,
    /// Value from a previous paginated response.
    /// Resume retrieving data where that response left off.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::nft_buy_offers")]
    pub command: RequestMethod,
}

impl<'a> Default for NftBuyOffers<'a> {
    fn default() -> Self {
        NftBuyOffers {
            nft_id: "",
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            marker: None,
            command: RequestMethod::NftBuyOffers,
        }
    }
}

impl<'a> Model for NftBuyOffers<'a> {}

impl<'a> NftBuyOffers<'a> {
    fn new(
        nft_id: &'a str,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        limit: Option<u16>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            nft_id,
            ledger_hash,
            ledger_index,
            limit,
            marker,
            command: RequestMethod::NftBuyOffers,
        }
    }
}
