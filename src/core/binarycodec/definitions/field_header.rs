//! A container struct for simultaneous storage of a
//! field's type code and field code.

use crate::utils::byte_conversions::ToBytes;

pub struct FieldHeader {
    pub type_code: i16,
    pub field_code: i16,
}

impl ToBytes for FieldHeader {
    fn to_be_bytes(&self) -> &[u8] {
        todo!()
    }

    fn to_le_bytes(&self) -> &[u8] {
        todo!()
    }

    fn to_bytes(&self) -> &[u8] {
        if cfg!(target_endian = "big") {
            self.to_be_bytes()
        } else {
            self.to_le_bytes()
        }
    }
}
