//! Context manager and helpers for the deserialization
//! of bytes into JSON.

use crate::core::binarycodec::binary_wrappers::binary_serializer::BinarySerializer;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::to_bytes;

pub trait BinaryParser {
    fn bp_peek(&self) -> Option<[u8; 1]>;
}

/// Peek the first byte of the BinaryParser.
impl BinaryParser for BinarySerializer {
    fn bp_peek(&self) -> Option<[u8; 1]> {
        if self.len() > 0 {
            Some(to_bytes!(self[0]))
        } else {
            None
        }
    }
}

pub fn bp_skip(bytes: &[u8], n: usize) -> Result<&[u8], XRPLBinaryCodecException> {
    if n > bytes.len() {
        Err(XRPLBinaryCodecException::new(&format!(
            "BinaryParser can't skip {} bytes, only contains {}.",
            n,
            bytes.len()
        )))
    } else {
        Ok(&bytes[n..])
    }
}

// pub bp_read(mut bytes: &[u8], n: usize) -> Result<&[u8], XRPLBinaryCodecException> {
// }
