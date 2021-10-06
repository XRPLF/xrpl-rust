/// Max length that can be represented in a single byte
/// per XRPL serialization encoding.
pub const MAX_SINGLE_BYTE_LENGTH: usize = 192;
/// Max length that can be represented in 2 bytes per
/// XRPL serialization encoding.
pub const MAX_DOUBLE_BYTE_LENGTH: usize = 12481;
/// Max value that can be used in the second byte of a
/// length field.
pub const MAX_SECOND_BYTE_VALUE: usize = 240;
/// Max value that can be represented in using two
/// 8-bit bytes (2^16)
pub const MAX_DOUBLE_BYTE_VALUE: usize = 65536;
/// Maximum length that can be encoded in a length
/// prefix per XRPL serialization encoding.
pub const MAX_LENGTH_VALUE: usize = 918744;
/// Max value that can be represented using one 8-bit
/// byte (2^8)
pub const MAX_BYTE_VALUE: usize = 256;
