use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    results::exceptions::XRPLResultException, XRPLModelException, XRPLModelResult,
};

use super::{XRPLResponse, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TxVersionMap<'a> {
    Default(Tx<'a>),
    V1(TxV1<'a>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxBase<'a> {
    /// The unique identifying hash of the transaction
    pub hash: Cow<'a, str>,
    /// The ledger index of the ledger that includes this transaction.
    pub ledger_index: Option<u32>,
    /// The transaction's compact transaction identifier.
    pub ctid: Option<Cow<'a, str>>,
    /// The close time of the ledger in which the transaction was applied, in seconds since the Ripple Epoch.
    pub date: Option<u32>,
    /// If true, this data comes from a validated ledger version; if omitted or set to false, this data is not final.
    pub validated: Option<bool>,
    /// (Deprecated) Alias for `ledger_index`
    #[serde(rename = "inLedger")]
    pub in_ledger: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx<'a> {
    #[serde(flatten)]
    pub base: TxBase<'a>,
    /// The transaction data represented in JSON.
    pub tx_json: Value,
    /// (JSON mode) Transaction metadata, which describes the results of the transaction.
    pub meta: Option<Value>, // TODO: replace with metadata as soon as implemented
    /// (Binary mode) Transaction metadata, which describes the results of the transaction, represented as a hex string.
    pub meta_blob: Option<Cow<'a, str>>,
    /// (Binary mode) The transaction data represented as a hex string.
    pub tx_blob: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxV1<'a> {
    #[serde(flatten)]
    pub base: TxBase<'a>,
    /// Transaction metadata, which describes the results of the transaction.
    pub meta: Option<TxMetaV1<'a>>, // TODO: replace with metadata as soon as implemented
    /// The transaction data represented as a hex string.
    pub tx: Option<Cow<'a, str>>,
    /// Other fields from the `Transaction` object
    #[serde(flatten)]
    pub tx_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TxMetaV1<'a> {
    Json(Value),
    Blob(Cow<'a, str>),
}

impl<'a> TryFrom<XRPLResult<'a>> for TxVersionMap<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Tx(tx) => Ok(tx),
            res => Err(
                XRPLResultException::UnexpectedResultType("Tx".to_string(), res.get_name()).into(),
            ),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for TxVersionMap<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => TxVersionMap::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for Tx<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::Tx(TxVersionMap::Default(tx)) => Ok(tx),
                res => Err(XRPLResultException::UnexpectedResultType(
                    "Tx (v2)".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for TxV1<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::Tx(TxVersionMap::V1(tx)) => Ok(tx),
                res => Err(XRPLResultException::UnexpectedResultType(
                    "Tx (v1)".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    const RESPONSE: &str = r#"{
        "tx_json": {
        "Account": "r3PDtZSa5LiYp1Ysn1vMuMzB59RzV3W9QH",
        "DeliverMax": {
            "currency": "USD",
            "issuer": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "value": "1"
        },
        "Destination": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
        "Fee": "10",
        "Flags": 0,
        "Paths": [
            [
                {
                    "account": "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
                    "currency": "USD",
                    "issuer": "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
                    "type": 49
                }
            ],
            [
                {
                    "account": "rD1jovjQeEpvaDwn9wKaYokkXXrqo4D23x",
                    "currency": "USD",
                    "issuer": "rD1jovjQeEpvaDwn9wKaYokkXXrqo4D23x",
                    "type": 49
                },
                {
                    "account": "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY",
                    "currency": "USD",
                    "issuer": "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY",
                    "type": 49
                },
                {
                    "account": "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
                    "currency": "USD",
                    "issuer": "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
                    "type": 49
                }
            ]
        ],
        "SendMax": {
            "currency": "USD",
            "issuer": "r3PDtZSa5LiYp1Ysn1vMuMzB59RzV3W9QH",
            "value": "1.01"
        },
        "Sequence": 88,
        "SigningPubKey": "02EAE5DAB54DD8E1C49641D848D5B97D1B29149106174322EDF98A1B2CCE5D7F8E",
        "TransactionType": "Payment",
        "TxnSignature": "30440220791B6A3E036ECEFFE99E8D4957564E8C84D1548C8C3E80A87ED1AA646ECCFB16022037C5CAC97E34E3021EBB426479F2ACF3ACA75DB91DCC48D1BCFB4CF547CFEAA0",
        "date": 416445410,
        "ledger_index": 348734
        },
        "ctid": "C005523E00000000",
        "hash": "E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7",
        "meta": {
        "AffectedNodes": [
            {
            "ModifiedNode": {
                "FinalFields": {
                "Account": "r3PDtZSa5LiYp1Ysn1vMuMzB59RzV3W9QH",
                "Balance": "59328999119",
                "Flags": 0,
                "OwnerCount": 11,
                "Sequence": 89
                },
                "LedgerEntryType": "AccountRoot",
                "LedgerIndex": "E0D7BDE68B468FF0B8D948FD865576517DA987569833A05374ADB9A72E870A06",
                "PreviousFields": {
                "Balance": "59328999129",
                "Sequence": 88
                },
                "PreviousTxnID": "C26AA6B4F7C3B9F55E17CD0D11F12032A1C7AD2757229FFD277C9447A8815E6E",
                "PreviousTxnLgrSeq": 348700
            }
            },
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
                "HighNode": "0",
                "LowLimit": {
                    "currency": "USD",
                    "issuer": "r3PDtZSa5LiYp1Ysn1vMuMzB59RzV3W9QH",
                    "value": "0"
                },
                "LowNode": "0"
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
        },
        "validated": true,
        "ledger_index": 348734,
        "ledger_hash": "195F62F34EB2CCFA4C5888BA20387E82EB353DDB4508BAE6A835AF19FB8B0C09",
        "close_time_iso": "2013-03-12T23:16:50Z"
    }"#;

    const RESPONSE_V1: &str = r#"{
        "Account": "rLQgQmY6sLmaj4syXStaBvBbbfV9EyraZu",
        "Domain": "6578616D706C652E636F6D",
        "Fee": "10",
        "Flags": 0,
        "LastLedgerSequence": 3282572,
        "Sequence": 3282552,
        "SigningPubKey": "ED3DC4D1235D789F4269F9EABAF122A63FDCEEF51355BE45E9D52D52D25E1174B4",
        "TransactionType": "AccountSet",
        "TxnSignature": "B75FA71C5411923A43B5ED9DCA836FAF30D76D485492A94A5B542FECC483F94EAF83A1CABD03DB76C77BE7FCD2D6CD5A873F0A448E8E5EF727D50EA8C7F84603",
        "hash": "AFEDEA4FBD3B36AB168900AAF503BF949A922E8A9F6F3542CD9B3413527DC87F",
        "status": "success",
        "validated": false
    }"#;

    #[test]
    fn test_deserialize_tx() -> XRPLModelResult<()> {
        let _: Tx = serde_json::from_str(RESPONSE).unwrap();
        let _: TxV1 = serde_json::from_str(RESPONSE_V1).unwrap();

        Ok(())
    }
}
