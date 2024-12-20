use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{requests::Marker, XRPLModelException, XRPLModelResult};

use super::{exceptions::XRPLResultException, NftOffer, XRPLResponse, XRPLResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTBuyOffer<'a> {
    pub offers: Vec<NftOffer<'a>>,
    pub nft_id: Cow<'a, str>,
    pub limit: Option<u32>,
    pub marker: Option<Marker<'a>>,
}

impl<'a> TryFrom<XRPLResult<'a>> for NFTBuyOffer<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTBuyOffer(nft_buy_offer) => Ok(nft_buy_offer),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTBuyOffer".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for NFTBuyOffer<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => NFTBuyOffer::try_from(result),
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
                "amount": "1500",
                "flags": 0,
                "nft_offer_index": "3212D26DB00031889D4EF7D9129BB0FA673B5B40B1759564486C0F0946BA203F",
                "owner": "rsuHaTvJh1bDmDoxX9QcKP7HEBSBt4XsHx"
            }
        ]
    }"#;

    #[test]
    fn test_deserialize_nft_buy_offer() -> XRPLModelResult<()> {
        let _: NFTBuyOffer = serde_json::from_str(RESPONSE)?;

        Ok(())
    }
}
