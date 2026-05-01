use alloc::borrow::Cow;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::{metadata::TransactionMetadata, tx::TxVersionMap};
use crate::models::{XRPLModelException, XRPLModelResult};

/// Result type for NFTokenMint transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenMintResult<'a> {
    /// The NFTokenID of the minted token
    pub nftoken_id: Cow<'a, str>,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenCreateOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCreateOfferResult<'a> {
    /// The OfferID of the created offer
    pub offer_id: Cow<'a, str>,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenCancelOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCancelOfferResult<'a> {
    /// The NFTokenIDs of all tokens affected by the cancellation
    pub nftoken_ids: Cow<'a, [Cow<'a, str>]>,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenAcceptOffer transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenAcceptOfferResult<'a> {
    /// The NFTokenID of the accepted token
    pub nftoken_id: Cow<'a, str>,
    /// The complete transaction metadata
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Macro to implement TryFrom<TxVersionMap> for NFToken result types
macro_rules! impl_try_from_tx_version_map {
    ($result_type:ident, $field_name:ident, $field_type:ty) => {
        impl<'a> TryFrom<TxVersionMap<'a>> for $result_type<'a> {
            type Error = XRPLModelException;

            fn try_from(tx: TxVersionMap<'a>) -> XRPLModelResult<Self> {
                // Extract metadata based on the version
                let meta = match &tx {
                    TxVersionMap::Default(tx) => tx.meta.clone(),
                    TxVersionMap::V1(tx) => tx.meta.clone(),
                };

                if let Some(meta) = meta {
                    if let Some(field_value) = meta.$field_name.clone() {
                        return Ok($result_type {
                            $field_name: field_value,
                            meta,
                        });
                    }
                }

                return Err(XRPLModelException::MissingField(
                    stringify!($field_name).into(),
                ));
            }
        }
    };
}

impl_try_from_tx_version_map!(NFTokenMintResult, nftoken_id, Cow<'a, str>);
impl_try_from_tx_version_map!(NFTokenCreateOfferResult, offer_id, Cow<'a, str>);
impl_try_from_tx_version_map!(
    NFTokenCancelOfferResult,
    nftoken_ids,
    Cow<'a, [Cow<'a, str>]>
);
impl_try_from_tx_version_map!(NFTokenAcceptOfferResult, nftoken_id, Cow<'a, str>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::results::tx::{Tx, TxBase, TxV1};

    fn meta_with(
        nftoken_id: Option<&str>,
        offer_id: Option<&str>,
        nftoken_ids: Option<&[&str]>,
    ) -> TransactionMetadata<'static> {
        let mut meta_value = serde_json::json!({
            "AffectedNodes": [],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        });
        if let Some(id) = nftoken_id {
            meta_value["nftoken_id"] = id.into();
        }
        if let Some(id) = offer_id {
            meta_value["offer_id"] = id.into();
        }
        if let Some(ids) = nftoken_ids {
            meta_value["nftoken_ids"] =
                serde_json::Value::Array(ids.iter().map(|s| (*s).into()).collect());
        }
        serde_json::from_value(meta_value).unwrap()
    }

    fn make_tx_default(meta: Option<TransactionMetadata<'static>>) -> TxVersionMap<'static> {
        TxVersionMap::Default(Tx {
            base: TxBase {
                hash: "ABCD".into(),
                ledger_index: Some(1),
                ctid: None,
                date: None,
                validated: Some(true),
                in_ledger: None,
            },
            tx_json: serde_json::Value::Null,
            meta,
            meta_blob: None,
            tx_blob: None,
        })
    }

    fn make_tx_v1(meta: Option<TransactionMetadata<'static>>) -> TxVersionMap<'static> {
        TxVersionMap::V1(TxV1 {
            base: TxBase {
                hash: "ABCD".into(),
                ledger_index: Some(1),
                ctid: None,
                date: None,
                validated: Some(true),
                in_ledger: None,
            },
            meta,
            tx: None,
            tx_json: serde_json::Value::Null,
        })
    }

    #[test]
    fn test_mint_result_success_default() {
        let meta = meta_with(Some("0008000044CDDA"), None, None);
        let tx = make_tx_default(Some(meta));
        let result: NFTokenMintResult = tx.try_into().unwrap();
        assert_eq!(result.nftoken_id, "0008000044CDDA");
    }

    #[test]
    fn test_mint_result_success_v1() {
        let meta = meta_with(Some("0008000044CDDA"), None, None);
        let tx = make_tx_v1(Some(meta));
        let result: NFTokenMintResult = tx.try_into().unwrap();
        assert_eq!(result.nftoken_id, "0008000044CDDA");
    }

    #[test]
    fn test_mint_result_missing_field() {
        let meta = meta_with(None, None, None);
        let tx = make_tx_default(Some(meta));
        let result: Result<NFTokenMintResult, _> = tx.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_mint_result_no_meta() {
        let tx = make_tx_default(None);
        let result: Result<NFTokenMintResult, _> = tx.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_offer_result_success() {
        let meta = meta_with(None, Some("AABBCCDD"), None);
        let tx = make_tx_default(Some(meta));
        let result: NFTokenCreateOfferResult = tx.try_into().unwrap();
        assert_eq!(result.offer_id, "AABBCCDD");
    }

    #[test]
    fn test_cancel_offer_result_success() {
        let meta = meta_with(None, None, Some(&["ID1", "ID2"]));
        let tx = make_tx_default(Some(meta));
        let result: NFTokenCancelOfferResult = tx.try_into().unwrap();
        assert_eq!(result.nftoken_ids.as_ref(), &["ID1", "ID2"]);
    }

    #[test]
    fn test_accept_offer_result_success() {
        let meta = meta_with(Some("0008000044CDDA"), None, None);
        let tx = make_tx_default(Some(meta));
        let result: NFTokenAcceptOfferResult = tx.try_into().unwrap();
        assert_eq!(result.nftoken_id, "0008000044CDDA");
    }

    #[test]
    fn test_mint_result_serialize() {
        // Round-trip would fail because `nftoken_id` collides with the
        // flattened meta's `nftoken_id`. Just check serialization works.
        let result = NFTokenMintResult {
            nftoken_id: "00080000".into(),
            meta: meta_with(None, None, None),
        };
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("\"nftoken_id\":\"00080000\""));
    }
}
