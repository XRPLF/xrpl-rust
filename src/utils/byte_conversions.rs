//! Conversion helpers for byte arrays.

/// Converter to byte array with endianness.
pub trait ToBytes {
    /// Return the byte array of self.
    fn to_bytes(&self) -> Vec<u8>;
}
