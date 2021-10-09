//! A container struct for simultaneous storage of a
//! field's type code and field code.

use crate::utils::byte_conversions::ToBytes;

#[derive(Debug, Clone)]
pub struct FieldHeader {
    pub type_code: i16,
    pub field_code: i16,
}

impl ToString for FieldHeader {
    fn to_string(&self) -> String {
        format!("{}_{}", self.type_code, self.field_code)
    }
}

impl ToBytes for FieldHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let mut header_bytes = vec![];

        if self.type_code < 16 {
            if self.field_code < 16 {
                let shift = self.type_code << 4 | self.field_code;
                header_bytes.extend_from_slice(&shift.to_be_bytes());

                header_bytes
            } else {
                let shift = self.type_code << 4;

                header_bytes.extend_from_slice(&shift.to_be_bytes());
                header_bytes.extend_from_slice(&self.field_code.to_be_bytes());

                header_bytes
            }
        } else if self.field_code < 16 {
            header_bytes.extend_from_slice(&self.field_code.to_be_bytes());
            header_bytes.extend_from_slice(&self.type_code.to_be_bytes());

            header_bytes
        } else {
            header_bytes.extend_from_slice(&[0]);
            header_bytes.extend_from_slice(&self.type_code.to_be_bytes());
            header_bytes.extend_from_slice(&self.field_code.to_be_bytes());

            header_bytes
        }
    }
}
