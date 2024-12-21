use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};

use crate::models::{XRPLModelException, XRPLModelResult};

use super::{exceptions::XRPLResultException, XRPLResponse, XRPLResult};

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

impl<'a> TryFrom<XRPLResult<'a>> for NFTInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTInfo(nft_info) => Ok(nft_info),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTInfo".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for NFTInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => NFTInfo::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
