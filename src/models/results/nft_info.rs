use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// See NFT Info:
/// `<https://xrpl.org/nft_sell_offers.html>`
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTInfo<'a> {
    #[serde(flatten)]
    pub base: NFToken<'a>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFToken<'a> {
    pub nft_id: Cow<'a, str>,
    pub ledger_index: u32,
    pub owner: Cow<'a, str>,
    pub is_burned: bool,
    pub flags: u32,
    pub tansfer_fee: u32,
    pub issuer: Cow<'a, str>,
    pub nft_taxon: u32,
    pub nft_serial: u32,
    pub uri: Cow<'a, str>,
}
