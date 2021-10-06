//! Conversion helpers for byte arrays.

/// Converter to byte array with endianness.
pub trait ToBytes {
    /// Return the byte array of self in big endian.
    fn to_be_bytes(&self) -> &[u8];

    /// Return the byte array of self in little endian.
    fn to_le_bytes(&self) -> &[u8];

    /// Return the byte array of self.
    fn to_bytes(&self) -> &[u8];
}
