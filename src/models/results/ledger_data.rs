use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::marker::Marker;

use crate::_serde::marker;

/// Response format for the ledger_data method, which retrieves contents of
/// the specified ledger.
///
/// See Ledger Data:
/// `<https://xrpl.org/ledger_data.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerData<'a> {
    /// The ledger index of this ledger version.
    pub ledger_index: u32,
    /// Unique identifying hash of this ledger version.
    pub ledger_hash: Cow<'a, str>,
    /// Array of JSON objects containing data from the ledger's state tree.
    pub state: Cow<'a, [LedgerObject<'a>]>,
    /// Server-defined value indicating the response is paginated.
    /// Pass this to the next call to resume where this call left off.
    #[serde(with = "marker", default)]
    pub marker: Option<Marker>,
}

/// Represents a single object in the ledger's state tree.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerObject<'a> {
    /// (Only included if binary true) Hex representation of the requested data
    pub data: Option<Cow<'a, str>>,
    /// (Only included if binary false) String indicating what type of ledger
    /// object this object represents.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: Option<Cow<'a, str>>,
    /// Unique identifier for this ledger entry, as hex.
    pub index: Cow<'a, str>,
    /// Additional fields describing this object, depending on which ledger
    /// object type it is.
    #[serde(flatten)]
    pub additional_fields: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_data_deserialize() {
        let json = r#"{
            "ledger_hash": "842B57C1CC0613299A686D3E9F310EC0422C84D3911E5056389AA7E5808A93C8",
            "ledger_index": 123456,
            "state": [
                {
                    "data": "1100612200000000240000000125000000012D00000000",
                    "index": "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B453"
                },
                {
                    "data": "1100612200000000240000000125000000012D00000001",
                    "index": "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B454"
                }
            ]
        }"#;

        let result: LedgerData = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.ledger_hash,
            "842B57C1CC0613299A686D3E9F310EC0422C84D3911E5056389AA7E5808A93C8"
        );
        assert_eq!(result.ledger_index, 123456);
        assert_eq!(result.state.len(), 2);

        // Test first state object
        let first_object = &result.state[0];
        assert_eq!(
            first_object.data.as_ref().unwrap(),
            "1100612200000000240000000125000000012D00000000"
        );
        assert_eq!(
            first_object.index,
            "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B453"
        );

        // Test second state object
        let second_object = &result.state[1];
        assert_eq!(
            second_object.data.as_ref().unwrap(),
            "1100612200000000240000000125000000012D00000001"
        );
        assert_eq!(
            second_object.index,
            "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B454"
        );
    }

    #[test]
    fn test_ledger_data_serialize() {
        let ledger_data = LedgerData {
            ledger_hash: "842B57C1CC0613299A686D3E9F310EC0422C84D3911E5056389AA7E5808A93C8".into(),
            ledger_index: 123456,
            state: alloc::vec![
                LedgerObject {
                    data: Some("1100612200000000240000000125000000012D00000000".into()),
                    ledger_entry_type: None,
                    index: "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B453"
                        .into(),
                    additional_fields: Some(serde_json::Map::new().into()),
                },
                LedgerObject {
                    data: Some("1100612200000000240000000125000000012D00000001".into()),
                    ledger_entry_type: None,
                    index: "00001A2969BE1FC85F1D7A55282FA2E6D95C71D2E3DC764A53075D1B4FE31B454"
                        .into(),
                    additional_fields: Some(serde_json::Map::new().into()),
                },
            ]
            .into(),
            marker: None,
        };

        let serialized = serde_json::to_string(&ledger_data).unwrap();
        let deserialized: LedgerData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ledger_data, deserialized);
    }
}
