//! Conversion helpers for byte arrays.

use alloc::vec::Vec;

/// Converter to byte array with endianness.
pub trait ToBytes {
    /// Return the byte array of self.
    fn to_bytes(&self) -> Vec<u8>;
}
