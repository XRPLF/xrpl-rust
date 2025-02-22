use crate::models::{results::tx::Tx, XRPLModelException, XRPLModelResult};
use alloc::{string::String, vec::Vec};
use core::convert::TryFrom;
use serde::{Deserialize, Serialize};

/// Result type for NFTokenMint transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenMintResult {
    /// The NFTokenID of the minted token
    pub nftoken_id: String,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: serde_json::Value,
}

/// Result type for NFTokenCreateOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCreateOfferResult {
    /// The OfferID of the created offer
    pub offer_id: String,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: serde_json::Value,
}

/// Result type for NFTokenCancelOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCancelOfferResult {
    /// The NFTokenIDs of all tokens affected by the cancellation
    pub nftoken_ids: Vec<String>,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: serde_json::Value,
}

/// Result type for NFTokenAcceptOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenAcceptOfferResult {
    /// The NFTokenID of the accepted token
    pub nftoken_id: String,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: serde_json::Value,
}

impl<'a> TryFrom<Tx<'a>> for NFTokenMintResult {
    type Error = XRPLModelException;

    fn try_from(tx: Tx<'a>) -> XRPLModelResult<Self> {
        let nftoken_id = tx
            .meta
            .get("nftoken_id")
            .and_then(|id| id.as_str())
            .map(String::from)
            .ok_or(XRPLModelException::MissingField("nftoken_id".into()))?;

        Ok(NFTokenMintResult {
            nftoken_id,
            meta: tx.meta,
        })
    }
}

impl<'a> TryFrom<Tx<'a>> for NFTokenCreateOfferResult {
    type Error = XRPLModelException;

    fn try_from(tx: Tx<'a>) -> XRPLModelResult<Self> {
        let offer_id = tx
            .meta
            .get("offer_id")
            .and_then(|id| id.as_str())
            .map(String::from)
            .ok_or(XRPLModelException::MissingField("offer_id".into()))?;

        Ok(NFTokenCreateOfferResult {
            offer_id,
            meta: tx.meta,
        })
    }
}

impl<'a> TryFrom<Tx<'a>> for NFTokenCancelOfferResult {
    type Error = XRPLModelException;

    fn try_from(tx: Tx<'a>) -> XRPLModelResult<Self> {
        let nftoken_ids = tx
            .meta
            .get("nftoken_ids")
            .and_then(|ids| ids.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .ok_or(XRPLModelException::MissingField("nftoken_ids".into()))?;

        Ok(NFTokenCancelOfferResult {
            nftoken_ids,
            meta: tx.meta,
        })
    }
}

impl<'a> TryFrom<Tx<'a>> for NFTokenAcceptOfferResult {
    type Error = XRPLModelException;

    fn try_from(tx: Tx<'a>) -> XRPLModelResult<Self> {
        let nftoken_id = tx
            .meta
            .get("nftoken_id")
            .and_then(|id| id.as_str())
            .map(String::from)
            .ok_or(XRPLModelException::MissingField("nftoken_id".into()))?;

        Ok(NFTokenAcceptOfferResult {
            nftoken_id,
            meta: tx.meta,
        })
    }
}
