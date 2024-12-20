use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{requests::Marker, XRPLModelException, XRPLModelResult};

use super::{exceptions::XRPLResultException, nft_info::NFToken, XRPLResponse, XRPLResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTsByIssuer<'a> {
    pub issuer: Cow<'a, str>,
    pub nfts: Vec<NFToken<'a>>,
    pub marker: Option<Marker<'a>>,
    pub limit: Option<u32>,
    pub nft_taxon: Option<u32>,
}

impl<'a> TryFrom<XRPLResult<'a>> for NFTsByIssuer<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTsByIssuer(nfts_by_issuer) => Ok(nfts_by_issuer),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTsByIssuer".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for NFTsByIssuer<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => NFTsByIssuer::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
