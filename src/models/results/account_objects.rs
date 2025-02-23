use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::marker::Marker;
use crate::_serde::marker;

/// Response format for the account_objects method, which returns the raw
/// ledger format for all objects owned by an account.
///
/// See Account Objects:
/// `<https://xrpl.org/account_objects.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountObjects<'a> {
    /// Unique Address of the account this request corresponds to
    pub account: Cow<'a, str>,
    /// Array of objects owned by this account. Each object is in its raw
    /// ledger format. Using Value since objects can be of different types
    /// (RippleState, Offer, etc.)
    pub account_objects: Cow<'a, [Value]>,
    /// The identifying hash of the ledger that was used to generate this
    /// response. May be omitted.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version that was used to generate
    /// this response. May be omitted if ledger_current_index is provided.
    pub ledger_index: u32,
    /// The ledger index of the current in-progress ledger version, which was
    /// used to generate this response. May be omitted if ledger_hash or
    /// ledger_index is provided.
    pub ledger_current_index: Option<u32>,
    /// The limit that was used in this request, if any.
    pub limit: Option<u32>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off. Omitted when
    /// there are no additional pages after this one.
    #[serde(with = "marker", default)]
    pub marker: Option<Marker>,
    /// If true, the information in this response comes from a validated
    /// ledger version. Otherwise, the information is subject to change.
    pub validated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_objects_deserialization() {
        let json = r#"{
            "account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "account_objects": [
                {
                    "Balance": {
                        "currency": "ASP",
                        "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji",
                        "value": "0"
                    },
                    "Flags": 65536,
                    "HighLimit": {
                        "currency": "ASP",
                        "issuer": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
                        "value": "0"
                    },
                    "HighNode": "0000000000000000",
                    "LedgerEntryType": "RippleState",
                    "LowLimit": {
                        "currency": "ASP",
                        "issuer": "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z",
                        "value": "10"
                    },
                    "LowNode": "0000000000000000",
                    "PreviousTxnID": "BF7555B0F018E3C5E2A3FF9437A1A5092F32903BE246202F988181B9CED0D862",
                    "PreviousTxnLgrSeq": 1438879,
                    "index": "2243B0B630EA6F7330B654EFA53E27A7609D9484E535AB11B7F946DF3D247CE9"
                }
            ],
            "ledger_hash": "4C99E5F63C0D0B1C2283B4F5DCE2239F80CE92E8B1A6AED1E110C198FC96E659",
            "ledger_index": 14380380,
            "limit": 10,
            "marker": "F60ADF645E78B69857D2E4AEC8B7742FEABC8431BD8611D099B428C3E816DF93,94A9F05FEF9A153229E2E997E64919FD75AAE2028C8153E8EBDB4440BD3ECBB5",
            "validated": true
        }"#;

        let account_objects: AccountObjects = serde_json::from_str(json).unwrap();

        // Test main struct fields
        assert_eq!(
            account_objects.account,
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59"
        );
        assert_eq!(
            account_objects.ledger_hash.unwrap(),
            "4C99E5F63C0D0B1C2283B4F5DCE2239F80CE92E8B1A6AED1E110C198FC96E659"
        );
        assert_eq!(account_objects.ledger_index, 14380380);
        assert_eq!(account_objects.limit.unwrap(), 10);
        assert!(account_objects.validated);

        // Test account objects array
        assert_eq!(account_objects.account_objects.len(), 1);

        // Test marker
        let marker = account_objects.marker.unwrap();
        assert_eq!(marker.ledger, 46982);
        assert_eq!(marker.seq, 8835);

        // Test first account object
        let first_object = &account_objects.account_objects[0];
        assert_eq!(first_object["LedgerEntryType"], "RippleState");
        assert_eq!(first_object["Flags"], 65536);
        assert_eq!(
            first_object["PreviousTxnID"],
            "BF7555B0F018E3C5E2A3FF9437A1A5092F32903BE246202F988181B9CED0D862"
        );
    }
}
