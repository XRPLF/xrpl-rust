//! A collection of serialization information about
//! a specific field type.

use crate::core::binarycodec::definitions::field_header::FieldHeader;
use crate::core::binarycodec::definitions::field_info::FieldInfo;

/// A collection of serialization information about
/// a specific field type.
pub struct FieldInstance<'a> {
    pub nth: i16,
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing: bool,
    pub associated_type: &'a str,
    pub name: &'a str,
    pub header: FieldHeader,
    pub ordinal: i16,
}

impl<'a> FieldInstance<'a> {
    pub fn new(field_info: &'a FieldInfo, field_name: &'a str, field_header: FieldHeader) -> Self {
        FieldInstance {
            nth: field_info.nth,
            is_vl_encoded: field_info.is_vl_encoded,
            is_serialized: field_info.is_serialized,
            is_signing: field_info.is_signing_field,
            name: field_name,
            ordinal: &field_header.type_code << 16 | field_info.nth,
            header: field_header,
            associated_type: &field_info.r#type,
        }
    }
}
