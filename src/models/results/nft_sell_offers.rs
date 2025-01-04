use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{requests::Marker, XRPLModelException, XRPLModelResult};

use super::{exceptions::XRPLResultException, NftOffer, XRPLResponse, XRPLResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTSellOffer<'a> {
    pub offers: Vec<NftOffer<'a>>,
    pub nft_id: Cow<'a, str>,
    pub limit: Option<u32>,
    pub marker: Option<Marker<'a>>,
}

impl<'a> TryFrom<XRPLResult<'a>> for NFTSellOffer<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTSellOffers(nft_sell_offers) => Ok(nft_sell_offers),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTSellOffers".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for NFTSellOffer<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => NFTSellOffer::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    const RESPONSE: &str = r#"{
        "nft_id": "00090000D0B007439B080E9B05BF62403911301A7B1F0CFAA048C0A200000007",
        "offers": [
            {
                "amount": "1000",
                "flags": 1,
                "nft_offer_index": "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1",
                "owner": "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt"
            }
        ]
    }"#;

    #[test]
    fn test_deserialize_nft_buy_offer() -> XRPLModelResult<()> {
        let _: NFTSellOffer = serde_json::from_str(RESPONSE)?;

        Ok(())
    }
}
