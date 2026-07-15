use alloc::{borrow::Cow, vec::Vec};
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::tx::TxVersionMap;
use crate::models::transactions::metadata::TransactionMetadata;
use crate::models::{XRPLModelException, XRPLModelResult};

/// Result type for NFTokenMint transaction.
/// Access the minted token ID via `meta.nftoken_id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenMintResult<'a> {
    /// The complete transaction metadata (includes `nftoken_id`)
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenCreateOffer transaction.
/// Access the created offer ID via `meta.offer_id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCreateOfferResult<'a> {
    /// The complete transaction metadata (includes `offer_id`)
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenCancelOffer transaction.
/// Access the cancelled token IDs via `meta.nftoken_ids`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenCancelOfferResult<'a> {
    /// The complete transaction metadata (includes `nftoken_ids`)
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Result type for NFTokenAcceptOffer transaction.
/// Access the accepted token ID via `meta.nftoken_id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NFTokenAcceptOfferResult<'a> {
    /// The complete transaction metadata (includes `nftoken_id`)
    #[serde(flatten)]
    pub meta: TransactionMetadata<'a>,
}

/// Macro to implement TryFrom<TxVersionMap> for NFToken result types.
/// Validates that the expected metadata field is present, then wraps the
/// full metadata in the result struct so callers access it via `result.meta`.
macro_rules! impl_try_from_tx_version_map {
    ($result_type:ident, $field_name:ident) => {
        impl<'a> TryFrom<TxVersionMap<'a>> for $result_type<'a> {
            type Error = XRPLModelException;

            fn try_from(tx: TxVersionMap<'a>) -> XRPLModelResult<Self> {
                let meta = match &tx {
                    TxVersionMap::Default(tx) => tx.meta.clone(),
                    TxVersionMap::V1(tx) => tx.meta.clone(),
                };

                if let Some(meta) = meta {
                    if meta.$field_name.is_some() {
                        return Ok($result_type { meta });
                    }
                }

                Err(XRPLModelException::MissingField(
                    stringify!($field_name).into(),
                ))
            }
        }
    };
}

impl_try_from_tx_version_map!(NFTokenMintResult, nftoken_id);
impl_try_from_tx_version_map!(NFTokenCreateOfferResult, offer_id);
impl_try_from_tx_version_map!(NFTokenCancelOfferResult, nftoken_ids);
impl_try_from_tx_version_map!(NFTokenAcceptOfferResult, nftoken_id);

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
        assert_eq!(result.meta.nftoken_id.as_deref(), Some("0008000044CDDA"));
    }

    #[test]
    fn test_mint_result_success_v1() {
        let meta = meta_with(Some("0008000044CDDA"), None, None);
        let tx = make_tx_v1(Some(meta));
        let result: NFTokenMintResult = tx.try_into().unwrap();
        assert_eq!(result.meta.nftoken_id.as_deref(), Some("0008000044CDDA"));
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
        assert_eq!(result.meta.offer_id.as_deref(), Some("AABBCCDD"));
    }

    #[test]
    fn test_cancel_offer_result_success() {
        let meta = meta_with(None, None, Some(&["ID1", "ID2"]));
        let tx = make_tx_default(Some(meta));
        let result: NFTokenCancelOfferResult = tx.try_into().unwrap();
        assert_eq!(
            result.meta.nftoken_ids.as_ref().unwrap().as_slice(),
            &["ID1", "ID2"][..]
        );
    }

    #[test]
    fn test_accept_offer_result_success() {
        let meta = meta_with(Some("0008000044CDDA"), None, None);
        let tx = make_tx_default(Some(meta));
        let result: NFTokenAcceptOfferResult = tx.try_into().unwrap();
        assert_eq!(result.meta.nftoken_id.as_deref(), Some("0008000044CDDA"));
    }

    #[test]
    fn test_mint_result_serialize() {
        // nftoken_id is carried in meta; no key collision with flatten.
        let result = NFTokenMintResult {
            meta: meta_with(Some("00080000"), None, None),
        };
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("\"nftoken_id\":\"00080000\""));
    }
}
