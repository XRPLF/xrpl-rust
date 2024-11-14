//! Functions for encoding objects into the XRP Ledger's
//! canonical binary format and decoding them.

pub mod types;

use core::fmt::Display;

pub use self::types::*;

use crate::utils::ToBytes;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub const CODE_MIN_VALUE: i16 = 1;
pub const CODE_MAX_VALUE: i16 = u8::MAX as i16;

/// A container class for simultaneous storage of a field's
/// type code and field code.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::binarycodec::definitions::FieldHeader;
///
/// let field_header = FieldHeader {
///     type_code: -2,
///     field_code: 0,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FieldHeader {
    pub type_code: i16,
    pub field_code: i16,
}

/// A collection of serialization information about
/// a specific field type.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::binarycodec::definitions::FieldInfo;
/// use xrpl::core::binarycodec::definitions::FieldHeader;
/// use xrpl::core::binarycodec::definitions::FieldInstance;
///
/// let field_header: FieldHeader = FieldHeader {
///     type_code: -2,
///     field_code: 0,
/// };
///
/// let field_info: FieldInfo = FieldInfo {
///     nth: 0,
///     is_vl_encoded: false,
///     is_serialized: false,
///     is_signing_field: false,
///     r#type: "Unknown".to_string(),
/// };
///
/// let field_instance: FieldInstance =
///     FieldInstance::new(&field_info, "Generic", field_header);
/// ```
#[derive(Debug, Clone)]
pub struct FieldInstance {
    pub nth: i16,
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing: bool,
    pub associated_type: String,
    pub name: String,
    pub header: FieldHeader,
    pub ordinal: i32,
}

///Model object for field info metadata from the
/// "fields" section of definitions.json.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::binarycodec::definitions::FieldInfo;
///
/// let field_info = FieldInfo {
///     nth: 0,
///     is_vl_encoded: false,
///     is_serialized: false,
///     is_signing_field: false,
///     r#type: "Unknown".to_string(),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    /// The field code -- sort order position for
    /// fields of the same type.
    pub nth: i16,
    /// Whether the serialized length of this
    /// field varies.
    #[serde(rename = "isVLEncoded")]
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

impl FieldInstance {
    /// Create a new FieldInstance.
    pub fn new(field_info: &FieldInfo, field_name: &str, field_header: FieldHeader) -> Self {
        FieldInstance {
            nth: field_info.nth,
            is_vl_encoded: field_info.is_vl_encoded,
            is_serialized: field_info.is_serialized,
            is_signing: field_info.is_signing_field,
            name: field_name.to_string(),
            ordinal: &(field_header.type_code as i32) << 16 | field_info.nth as i32,
            header: field_header,
            associated_type: field_info.r#type.to_string(),
        }
    }
}

impl Display for FieldHeader {
    /// Convert the FieldHeader to a String.
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}_{}", self.type_code, self.field_code)
    }
}

impl ToBytes for FieldHeader {
    /// Convert the FieldHeader to a Vec<u8>.
    fn to_bytes(&self) -> Vec<u8> {
        let mut header_bytes = vec![];

        if self.type_code < 16 {
            if self.field_code < 16 {
                let shift = (self.type_code << 4 | self.field_code) as u8;
                header_bytes.extend_from_slice(&shift.to_be_bytes());
            } else {
                let shift = (self.type_code << 4) as u8;

                header_bytes.extend_from_slice(&shift.to_be_bytes());
                header_bytes.extend_from_slice(&(self.field_code as u8).to_be_bytes());
            }
        } else if self.field_code < 16 {
            header_bytes.extend_from_slice(&(self.field_code as u8).to_be_bytes());
            header_bytes.extend_from_slice(&(self.type_code as u8).to_be_bytes());
        } else {
            header_bytes.extend_from_slice(&[0]);
            header_bytes.extend_from_slice(&(self.type_code as u8).to_be_bytes());
            header_bytes.extend_from_slice(&(self.field_code as u8).to_be_bytes());
        }

        header_bytes
    }
}
