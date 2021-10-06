//! Model object for field info from the "fields"
//! section of definitions.json.

use serde::{Deserialize, Serialize};

///Model object for field info metadata from the
/// "fields" section of definitions.json.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    /// The field code -- sort order position for
    /// fields of the same type.
    pub nth: i16,
    /// Whether the serialized length of this
    /// field varies.
    pub is_vl_encoded: bool,
    /// If the field is presented in binary
    /// serialized representation.
    pub is_serialized: bool,
    /// If the field should be included in signed
    /// transactions.
    pub is_signing_field: bool,
    /// The name of this field's serialization type,
    /// e.g. UInt32, AccountID, etc.
    pub r#type: String,
}
