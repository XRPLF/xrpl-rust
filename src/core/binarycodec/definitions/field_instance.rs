//! A collection of serialization information about
//! a specific field type.

use crate::core::binarycodec::definitions::field_header::FieldHeader;
use crate::core::binarycodec::definitions::field_info::FieldInfo;
use alloc::string::String;
use alloc::string::ToString;

/// A collection of serialization information about
/// a specific field type.
#[derive(Debug)]
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

impl FieldInstance {
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
