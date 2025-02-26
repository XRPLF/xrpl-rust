use alloc::string::ToString;
use alloc::{borrow::Cow, vec::Vec};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::requests::Marker;
use crate::models::{XRPLModelException, XRPLModelResult};

use super::{
    exceptions::XRPLResultException, metadata::TransactionMetadata, XRPLResponse, XRPLResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AccountTxVersionMap<'a> {
    Default(AccountTx<'a>),
    V1(AccountTxV1<'a>),
}

impl Default for AccountTxVersionMap<'_> {
    fn default() -> Self {
        Self::Default(AccountTx::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountTxBase<'a, T> {
    /// Unique Address identifying the related account
    pub account: Cow<'a, str>,
    /// The ledger index of the earliest ledger actually searched for
    /// transactions.
    pub ledger_index_min: Option<u32>,
    /// The ledger index of the most recent ledger actually searched for
    /// transactions.
    pub ledger_index_max: Option<u32>,
    /// The limit value used in the request. (This may differ from the actual
    /// limit value enforced by the server.)
    pub limit: Option<u16>,
    /// Array of transactions matching the request's criteria, as explained
    /// below.
    pub transactions: Vec<T>,
    /// If included and set to true, the information in this response comes
    /// from a validated ledger version. Otherwise, the information is
    /// subject to change.
    pub validated: Option<bool>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off.
    pub marker: Option<Marker<'a>>,
}

/// Response from an account_tx request, containing information about
/// transactions related to a specific account.
///
/// See Account TX:
/// `<https://xrpl.org/account_tx.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountTx<'a> {
    #[serde(flatten)]
    pub base: AccountTxBase<'a, AccountTxTransaction<'a>>,
    /// (JSON mode) The transaction results metadata in JSON.
    pub meta: Option<TransactionMetadata<'a>>,
    /// (Binary mode) The transaction results metadata as a hex string.
    pub meta_blob: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountTxV1<'a> {
    #[serde(flatten)]
    pub base: AccountTxBase<'a, AccountTxTransactionV1<'a>>,
    /// If binary is true, then this is a hex string of the transaction
    /// results metadata. Otherwise, the transaction results metadata is
    /// included in JSON format.
    pub meta: Option<TransactionMetadata<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TransactionBase<'a> {
    /// The ledger index of the ledger version that included this transaction.
    pub ledger_index: u32,
    /// Whether or not the transaction is included in a validated ledger. Any
    /// transaction not yet in a validated ledger is subject to change.
    pub validated: bool,
    /// (Binary mode) A unique hex string defining the transaction.
    pub tx_blob: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountTxTransaction<'a> {
    #[serde(flatten)]
    pub base: TransactionBase<'a>,
    /// The ledger close time represented in ISO 8601 time format.
    pub close_time_iso: Cow<'a, str>,
    /// The unique hash identifier of the transaction.
    pub hash: Cow<'a, str>,
    /// A hex string of the ledger version that included this transaction.
    pub ledger_hash: Cow<'a, str>,
    /// (JSON mode) JSON object defining the transaction.
    pub tx_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountTxTransactionV1<'a> {
    #[serde(flatten)]
    pub base: TransactionBase<'a>,
    /// (Binary mode) A hex string of the transaction in binary format.
    pub tx: Cow<'a, str>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountTxVersionMap<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::AccountTx(account_tx) => Ok(account_tx),
            res => Err(XRPLResultException::UnexpectedResultType(
                "AccountTx".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for AccountTxVersionMap<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => AccountTxVersionMap::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    const RESPONSE: &str = r#"{
            "account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
            "ledger_index_min": 32570,
            "ledger_index_max": 91824401,
            "transactions": [
                {
                    "meta": {
                    "AffectedNodes": [
                        {
                        "ModifiedNode": {
                            "FinalFields": {
                                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                                "AccountTxnID": "932CC7E9BAC1F7B9FA5381679F293EEC0A646E5E7F2F6D14C85FEE2102F0E66C",
                                "Balance": "1086222646",
                                "Domain": "6D64756F31332E636F6D",
                                "EmailHash": "98B4375E1D753E5B91627516F6D70977",
                                "Flags": 9568256,
                                "MessageKey": "0000000000000000000000070000000300",
                                "OwnerCount": 17,
                                "RegularKey": "rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ",
                                "Sequence": 393,
                                "TicketCount": 5,
                                "TransferRate": 4294967295
                            },
                            "LedgerEntryType": "AccountRoot",
                            "LedgerIndex": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8",
                            "PreviousFields": {
                            "Balance": "1086222601"
                            },
                            "PreviousTxnID": "7E50969CDEF8E12B1AD26E64B338935813624A4D1CDDC4C9457832524F0FF74C",
                            "PreviousTxnLgrSeq": 89353048
                        }
                        },
                        {
                        "ModifiedNode": {
                            "FinalFields": {
                            "Account": "rPJARH5nLWQisdmvDAbvzwS7N32Z1kusTZ",
                            "Balance": "55022190",
                            "Flags": 0,
                            "OwnerCount": 0,
                            "Sequence": 89113341
                            },
                            "LedgerEntryType": "AccountRoot",
                            "LedgerIndex": "C0363F86E070B70E7DA129736C3B05E509261C8668F61A7E958C4C10F17EAB90",
                            "PreviousFields": {
                            "Balance": "55022245",
                            "Sequence": 89113340
                            },
                            "PreviousTxnID": "60D0FE881F9B1457FB1711011C6E490C22532B1D495557D6488BE3A634167CEE",
                            "PreviousTxnLgrSeq": 90136515
                        }
                        }
                    ],
                    "TransactionIndex": 2,
                    "TransactionResult": "tesSUCCESS",
                    "delivered_amount": "45"
                    },
                    "tx_json": {
                        "Account": "rPJARH5nLWQisdmvDAbvzwS7N32Z1kusTZ",
                        "DeliverMax": "45",
                        "Destination": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                        "DestinationTag": 316562,
                        "Fee": "10",
                        "Sequence": 89113340,
                        "SigningPubKey": "EDE21591E615E1D77C8C8A7F95372D001B3DF090AB47B99729CFCBC1E4E07D35F4",
                        "TransactionType": "Payment",
                        "TxnSignature": "D229FEB6ED82367102AC12DE5045BE6D548CBB52E0CB8F037A23171910A6158FA3377F5118B6CEAFDB07D6D43F76FE29CC26BE1ACBC7A86C9D86E14043C66104",
                        "ledger_index": 90136515,
                        "date": 777284672
                    },
                    "ledger_index": 90136515,
                    "hash": "894541402AC968C98C329A88D097170B14BF4DEB8B2A7DF377EE89DDD332E018",
                    "ledger_hash": "14110F60753176E1F6A71AA084B6AD8663CBB46193CCFCDFAC02561626AA6B75",
                    "close_time_iso": "2024-08-18T08:24:32Z",
                    "validated": true
                },
                {
                    "meta": {
                    "AffectedNodes": [
                        {
                        "ModifiedNode": {
                            "FinalFields": {
                                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                                "AccountTxnID": "932CC7E9BAC1F7B9FA5381679F293EEC0A646E5E7F2F6D14C85FEE2102F0E66C",
                                "Balance": "1086222601",
                                "Domain": "6D64756F31332E636F6D",
                                "EmailHash": "98B4375E1D753E5B91627516F6D70977",
                                "Flags": 9568256,
                                "MessageKey": "0000000000000000000000070000000300",
                                "OwnerCount": 17,
                                "RegularKey": "rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ",
                                "Sequence": 393,
                                "TicketCount": 5,
                                "TransferRate": 4294967295
                            },
                            "LedgerEntryType": "AccountRoot",
                            "LedgerIndex": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8",
                            "PreviousFields": {
                                "Balance": "1086222552"
                            },
                            "PreviousTxnID": "EED9EB1880B951FAB3EE0DBBEB67B7ABEE3FA77F15782B6BD40342B3C23CFB75",
                            "PreviousTxnLgrSeq": 89343389
                        }
                        },
                        {
                        "ModifiedNode": {
                            "FinalFields": {
                                "Account": "rPSDqHdMPsnkmyUX4BvBkY8rycQYwrhUqw",
                                "Balance": "52611432",
                                "Flags": 0,
                                "OwnerCount": 0,
                                "Sequence": 89196186
                            },
                            "LedgerEntryType": "AccountRoot",
                            "LedgerIndex": "20761D2C37004C70318F7A3C5A1C35817A90A0AE56485F6E3281FB2B3F05B0C9",
                            "PreviousFields": {
                                "Balance": "52611491",
                                "Sequence": 89196185
                            },
                            "PreviousTxnID": "BAF86C2776C08407E0FAF42D374874E10430CB8C23AD464D9D9097EA326ABE92",
                            "PreviousTxnLgrSeq": 89353024
                        }
                        }
                    ],
                    "TransactionIndex": 4,
                    "TransactionResult": "tesSUCCESS",
                    "delivered_amount": "49"
                    },
                    "tx_json": {
                        "Account": "rPSDqHdMPsnkmyUX4BvBkY8rycQYwrhUqw",
                        "DeliverMax": "49",
                        "Destination": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                        "DestinationTag": 342662134,
                        "Fee": "10",
                        "Sequence": 89196185,
                        "SigningPubKey": "ED7E4A2970ADFCCE93D59D469322745E98CBEB3D7D5388728B3BB2268E71F30B0F",
                        "TransactionType": "Payment",
                        "TxnSignature": "8CE14FD18BD186694DED8C204C3FCC2A527CC24AD51C2E0B2B792D035C85D662BC1A1450A8DF04BBEC66821B362056311127C627056AC7779B385517FD3A9202",
                        "ledger_index": 89353048,
                        "date": 774249571
                    },
                    "ledger_index": 89353048,
                    "hash": "7E50969CDEF8E12B1AD26E64B338935813624A4D1CDDC4C9457832524F0FF74C",
                    "ledger_hash": "ED54DA98F3E495C36C2B0D9A511565E04454A1F4503B9DEE3FD39301D7625865",
                    "close_time_iso": "2024-07-14T05:19:31Z",
                    "validated": true
                }
            ],
            "validated": true,
            "marker": {
                "ledger": 89353048,
                "seq": 4
            },
            "limit": 2
    }"#;

    #[test]
    fn test_deserialize_account_tx() -> XRPLModelResult<()> {
        let _: AccountTxVersionMap = serde_json::from_str(RESPONSE)?;

        Ok(())
    }
}
