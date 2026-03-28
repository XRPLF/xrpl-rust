use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::ledger::objects::LedgerEntryType;
use crate::models::requests::LedgerIndex;
use crate::models::{Amount, IssuedCurrencyAmount, Model, ValidateCurrencies};

#[skip_serializing_none]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMetadata<'a> {
    #[serde(rename = "NFToken")]
    pub nftoken: NFTokenMetadataFields<'a>,
}

impl Model for NFTokenMetadata<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;
        Ok(())
    }
}

#[skip_serializing_none]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMetadataFields<'a> {
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    #[serde(rename = "URI")]
    pub uri: Cow<'a, str>,
}

impl Model for NFTokenMetadataFields<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;
        Ok(())
    }
}

#[skip_serializing_none]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct Fields<'a> {
    pub account: Option<Cow<'a, str>>,
    pub balance: Option<Amount<'a>>,
    pub book_directory: Option<Cow<'a, str>>,
    pub expiration: Option<u32>,
    #[serde(default)]
    pub flags: u32,
    pub low_limit: Option<IssuedCurrencyAmount<'a>>,
    pub high_limit: Option<IssuedCurrencyAmount<'a>>,
    pub next_page_min: Option<Cow<'a, str>>,
    #[serde(rename = "NFTokens")]
    pub nftokens: Option<Vec<NFTokenMetadata<'a>>>,
    pub previous_page_min: Option<Cow<'a, str>>,
    #[serde(default)]
    pub sequence: u32,
    pub taker_gets: Option<Amount<'a>>,
    pub taker_pays: Option<Amount<'a>>,
    pub xchain_claim_id: Option<Cow<'a, str>>,
}

impl Model for Fields<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;
        Ok(())
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum AffectedNode<'a> {
    #[serde(rename_all = "PascalCase")]
    CreatedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        new_fields: Fields<'a>,
    },
    #[serde(rename_all = "PascalCase")]
    ModifiedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        final_fields: Option<Fields<'a>>,
        previous_fields: Option<Fields<'a>>,
        previous_txn_id: Option<Cow<'a, str>>,
        previous_txn_lgr_seq: Option<u32>,
    },
    #[serde(rename_all = "PascalCase")]
    DeletedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        final_fields: Fields<'a>,
        previous_fields: Option<Fields<'a>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    CreatedNode,
    ModifiedNode,
    DeletedNode,
}

/// The amount actually delivered by a Payment transaction.
///
/// Can be an XRP drops string, an issued currency object, or the literal
/// string `"unavailable"` for partial payments before 2014-01-20.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DeliveredAmount<'a> {
    /// The literal string `"unavailable"` for partial payments before
    /// 2014-01-20. Must be first variant so serde tries it before
    /// `Amount::XRPAmount` which would match any string.
    #[serde(deserialize_with = "deserialize_unavailable")]
    Unavailable,
    Amount(Amount<'a>),
}

fn deserialize_unavailable<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    if s == "unavailable" {
        Ok(())
    } else {
        Err(serde::de::Error::custom("expected \"unavailable\""))
    }
}

impl<'a> DeliveredAmount<'a> {
    /// Returns the inner `Amount` if this is not `"unavailable"`.
    pub fn as_amount(&self) -> Option<&Amount<'a>> {
        match self {
            DeliveredAmount::Amount(a) => Some(a),
            DeliveredAmount::Unavailable => None,
        }
    }

    /// Returns true if the delivered amount is unavailable (pre-2014 partial payment).
    pub fn is_unavailable(&self) -> bool {
        matches!(self, DeliveredAmount::Unavailable)
    }
}

/// Transaction metadata describing the results of a transaction.
///
/// See Metadata:
/// `<https://xrpl.org/docs/references/protocol/transactions/metadata>`
#[skip_serializing_none]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMetadata<'a> {
    /// Array of objects describing changes to ledger entries this
    /// transaction made.
    pub affected_nodes: Vec<AffectedNode<'a>>,
    /// The transaction's position within the ledger that included it.
    pub transaction_index: u32,
    /// The transaction's result code (e.g. "tesSUCCESS").
    pub transaction_result: Cow<'a, str>,
    /// The currency amount actually delivered to the destination for Payment
    /// transactions. May be `"unavailable"` for partial payments before
    /// 2014-01-20.
    #[serde(rename = "delivered_amount")]
    pub delivered_amount: Option<DeliveredAmount<'a>>,
    /// NFTokenID for NFTokenMint and NFTokenAcceptOffer transactions.
    #[serde(rename = "nftoken_id")]
    pub nftoken_id: Option<Cow<'a, str>>,
    /// Array of NFTokenIDs for NFTokenCancelOffer transactions.
    #[serde(rename = "nftoken_ids")]
    pub nftoken_ids: Option<Vec<Cow<'a, str>>>,
    /// OfferID for NFTokenCreateOffer transactions.
    #[serde(rename = "offer_id")]
    pub offer_id: Option<Cow<'a, str>>,
    /// MPTokenIssuanceID for MPTokenIssuanceCreate transactions.
    #[serde(rename = "mpt_issuance_id")]
    pub mpt_issuance_id: Option<Cow<'a, str>>,
}

impl Default for TransactionMetadata<'_> {
    fn default() -> Self {
        Self {
            affected_nodes: Vec::new(),
            transaction_index: 0,
            transaction_result: Cow::Borrowed(""),
            delivered_amount: None,
            nftoken_id: None,
            nftoken_ids: None,
            offer_id: None,
            mpt_issuance_id: None,
        }
    }
}

impl Model for TransactionMetadata<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;
        Ok(())
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_deserialize_deleted_node() {
        let json = r#"
            {
                "DeletedNode": {
                    "FinalFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "BookDirectory": "623C4C4AD65873DA787AC85A0A1385FE6233B6DE100799474F20E441AE211B08",
                        "BookNode": "0",
                        "Flags": 0,
                        "OwnerNode": "0",
                        "PreviousTxnID": "3C5524789C20AE661CF4985EF694F11128FE267D084CB5E77DBB5FFD78E31E1C",
                        "PreviousTxnLgrSeq": 92173580,
                        "Sequence": 29337056,
                        "TakerGets": "17250658754",
                        "TakerPays": {
                            "currency": "CNY",
                            "issuer": "rJ1adrpGS3xsnQMb9Cw54tWJVFPuSdZHK",
                            "value": "159709.5313591656"
                        }
                    },
                    "LedgerEntryType": "Offer",
                    "LedgerIndex": "D11F69DE8A8CACB130F2E2B9893E5C97B9EE4136759C66C1F3497C8575FF5ED0"
                }
            }
        "#;
        let deleted_node = serde_json::from_str::<AffectedNode>(json);
        assert!(deleted_node.is_ok());
    }

    #[test]
    fn test_deserialize_modified_node() {
        let json = r#"
            {
                "ModifiedNode": {
                    "FinalFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "Balance": "5000542889",
                        "Flags": 0,
                        "OwnerCount": 5,
                        "Sequence": 29337064
                    },
                    "LedgerEntryType": "AccountRoot",
                    "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                    "PreviousFields": {
                        "Balance": "5000542904",
                        "Sequence": 29337063
                    },
                    "PreviousTxnID": "960FAFAF9CA0465B7475F888946F0D58F9CF49B18F3991D826B03A5025368DDE",
                    "PreviousTxnLgrSeq": 92173588
                }
            }
        "#;
        let modified_node = serde_json::from_str::<AffectedNode>(json);
        assert!(modified_node.is_ok());
    }

    #[test]
    fn test_deserialize_created_node() {
        let json = r#"
            {
                "CreatedNode": {
                    "LedgerEntryType": "AccountRoot",
                    "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                    "NewFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "Balance": "5000542889",
                        "Flags": 0,
                        "OwnerCount": 5,
                        "Sequence": 29337064
                    }
                }
            }
        "#;
        let created_node = serde_json::from_str::<AffectedNode>(json);
        assert!(created_node.is_ok());
    }

    #[test]
    fn test_deserialize_metadata_with_xrp_delivered_amount() {
        let json = r#"{
            "AffectedNodes": [],
            "TransactionIndex": 2,
            "TransactionResult": "tesSUCCESS",
            "delivered_amount": "45"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.transaction_index, 2);
        assert_eq!(meta.transaction_result, "tesSUCCESS");
        assert!(meta
            .delivered_amount
            .as_ref()
            .unwrap()
            .as_amount()
            .is_some());
    }

    #[test]
    fn test_deserialize_metadata_with_unavailable_delivered_amount() {
        let json = r#"{
            "AffectedNodes": [],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS",
            "delivered_amount": "unavailable"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert!(meta.delivered_amount.as_ref().unwrap().is_unavailable());
    }

    #[test]
    fn test_deserialize_metadata_with_iou_delivered_amount() {
        let json = r#"{
            "AffectedNodes": [],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS",
            "delivered_amount": {
                "currency": "USD",
                "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                "value": "1.5"
            }
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert!(meta
            .delivered_amount
            .as_ref()
            .unwrap()
            .as_amount()
            .is_some());
    }

    #[test]
    fn test_deserialize_metadata_with_nftoken_fields() {
        let json = r#"{
            "AffectedNodes": [],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS",
            "nftoken_id": "00080000B4F4AFC5984261F6D1A034BA3CE3B4ECB47E2B4B00000004",
            "offer_id": "68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C063E"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert!(meta.nftoken_id.is_some());
        assert!(meta.offer_id.is_some());
    }

    #[test]
    fn test_deserialize_full_metadata_from_results() {
        let json = r#"{
            "AffectedNodes": [
                {
                    "ModifiedNode": {
                        "FinalFields": {
                            "Account": "rBTwLga3i2gz3doX6Gva3MgEV8ZCD8jjah",
                            "Balance": "27724423128",
                            "Flags": 0,
                            "OwnerCount": 14,
                            "Sequence": 129693478
                        },
                        "LedgerEntryType": "AccountRoot",
                        "LedgerIndex": "1ED8DDFD80F275CB1CE7F18BB9D906655DE8029805D8B95FB9020B30425821EB",
                        "PreviousFields": {
                            "Balance": "27719423228",
                            "Sequence": 129693477
                        },
                        "PreviousTxnID": "3110F983CDC090750B45C9BFB74B8CE629CA80F57C35612402B2760153822BA5",
                        "PreviousTxnLgrSeq": 86724072
                    }
                },
                {
                    "DeletedNode": {
                        "FinalFields": {
                            "Account": "rPx6Rbh8fStXeP3LwECBisownN2ZyMyzYS",
                            "BookDirectory": "DFA3B6DDAB58C7E8E5D944E736DA4B7046C30E4F460FD9DE4E1566CBCC208000",
                            "BookNode": "0",
                            "Flags": 0,
                            "OwnerNode": "0",
                            "PreviousTxnID": "DCB061EC44BBF73BBC20CE0432E9D8D7C4B8B28ABA8AE5A5BA687476E7A796EF",
                            "PreviousTxnLgrSeq": 86724050,
                            "Sequence": 86586865,
                            "TakerGets": "0",
                            "TakerPays": {
                                "currency": "USD",
                                "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                                "value": "0"
                            }
                        },
                        "LedgerEntryType": "Offer",
                        "LedgerIndex": "348AF66EBD872FBF2BD23085D3FB4A200E15509451475027C4A5EE8D8B77C623"
                    }
                }
            ],
            "TransactionIndex": 5,
            "TransactionResult": "tesSUCCESS"
        }"#;

        let metadata: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.transaction_index, 5);
        assert_eq!(metadata.transaction_result, "tesSUCCESS");
        assert_eq!(metadata.affected_nodes.len(), 2);
    }

    #[test]
    fn test_get_errors_nftoken_metadata() {
        let meta = NFTokenMetadata {
            nftoken: NFTokenMetadataFields {
                nftoken_id: Cow::Borrowed(
                    "00080000B4F4AFC5984261F6D1A034BA3CE3B4ECB47E2B4B00000004",
                ),
                uri: Cow::Borrowed(
                    "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
                ),
            },
        };
        assert!(meta.get_errors().is_ok());
    }

    #[test]
    fn test_get_errors_nftoken_metadata_fields() {
        let fields = NFTokenMetadataFields {
            nftoken_id: Cow::Borrowed("00080000B4F4AFC5984261F6D1A034BA3CE3B4ECB47E2B4B00000004"),
            uri: Cow::Borrowed("ipfs://example"),
        };
        assert!(fields.get_errors().is_ok());
    }

    #[test]
    fn test_get_errors_fields() {
        let fields = Fields {
            account: Some(Cow::Borrowed("rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV")),
            balance: None,
            book_directory: None,
            expiration: None,
            flags: 0,
            low_limit: None,
            high_limit: None,
            next_page_min: None,
            nftokens: None,
            previous_page_min: None,
            sequence: 1,
            taker_gets: None,
            taker_pays: None,
            xchain_claim_id: None,
        };
        assert!(fields.get_errors().is_ok());
    }

    #[test]
    fn test_get_errors_transaction_metadata() {
        let meta = TransactionMetadata {
            affected_nodes: Vec::new(),
            transaction_index: 0,
            transaction_result: Cow::Borrowed("tesSUCCESS"),
            delivered_amount: None,
            nftoken_id: None,
            nftoken_ids: None,
            offer_id: None,
            mpt_issuance_id: None,
        };
        assert!(meta.get_errors().is_ok());
    }

    #[test]
    fn test_default_transaction_metadata() {
        let meta = TransactionMetadata::default();
        assert!(meta.affected_nodes.is_empty());
        assert_eq!(meta.transaction_index, 0);
        assert_eq!(meta.transaction_result, "");
        assert!(meta.delivered_amount.is_none());
        assert!(meta.nftoken_id.is_none());
        assert!(meta.nftoken_ids.is_none());
        assert!(meta.offer_id.is_none());
        assert!(meta.mpt_issuance_id.is_none());
    }

    #[test]
    fn test_delivered_amount_as_amount_unavailable() {
        let da = DeliveredAmount::Unavailable;
        assert!(da.as_amount().is_none());
        assert!(da.is_unavailable());
    }

    #[test]
    fn test_delivered_amount_as_amount_xrp() {
        let da = DeliveredAmount::Amount(Amount::XRPAmount(crate::models::XRPAmount(
            Cow::Borrowed("1000000"),
        )));
        assert!(da.as_amount().is_some());
        assert!(!da.is_unavailable());
    }

    #[test]
    fn test_deserialize_unavailable_rejects_non_unavailable_string() {
        // A non-"unavailable" string should NOT deserialize as DeliveredAmount::Unavailable
        // It should fall through to Amount::XRPAmount in the untagged enum
        let json = r#""some_other_string""#;
        let da: DeliveredAmount = serde_json::from_str(json).unwrap();
        // "some_other_string" is not "unavailable", so it matches Amount::XRPAmount
        assert!(!da.is_unavailable());
    }

    #[test]
    fn test_deserialize_metadata_with_ripple_state() {
        let json = r#"{
            "AffectedNodes": [
                {
                    "ModifiedNode": {
                        "FinalFields": {
                            "Balance": {
                                "currency": "USD",
                                "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji",
                                "value": "-1"
                            },
                            "Flags": 131072,
                            "HighLimit": {
                                "currency": "USD",
                                "issuer": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
                                "value": "100"
                            },
                            "LowLimit": {
                                "currency": "USD",
                                "issuer": "r3PDtZSa5LiYp1Ysn1vMuMzB59RzV3W9QH",
                                "value": "0"
                            }
                        },
                        "LedgerEntryType": "RippleState",
                        "LedgerIndex": "EA4BF03B4700123CDFFB6EB09DC1D6E28D5CEB7F680FB00FC24BC1C3BB2DB959",
                        "PreviousFields": {
                            "Balance": {
                                "currency": "USD",
                                "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji",
                                "value": "0"
                            }
                        },
                        "PreviousTxnID": "53354D84BAE8FDFC3F4DA879D984D24B929E7FEB9100D2AD9EFCD2E126BCCDC8",
                        "PreviousTxnLgrSeq": 343570
                    }
                }
            ],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS",
            "delivered_amount": "unavailable"
        }"#;

        let metadata: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.transaction_result, "tesSUCCESS");
        assert!(metadata.delivered_amount.as_ref().unwrap().is_unavailable());
    }
}
