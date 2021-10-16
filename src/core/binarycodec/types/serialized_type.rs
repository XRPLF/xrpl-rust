//! The base class for all binary codec field types.

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Contains a serialized buffer of a Serializer type.
pub struct SerializedType(Vec<u8>);

pub trait Serializable {
    /// Create a new instance of a type.
    fn new(buffer: Option<&[u8]>) -> Self;
    /// Construct a type from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Self, XRPLBinaryCodecException>
    where
        Self: Sized;
}

pub trait Buffered {
    /// Return the byte value.
    fn get_buffer(&self) -> &[u8];
}

impl<'value, T> From<T> for SerializedType
where
    T: Buffered,
{
    fn from(instance: T) -> Self {
        SerializedType(instance.get_buffer().to_vec())
    }
}

impl ToString for dyn Buffered {
    fn to_string(&self) -> String {
        hex::encode(self.get_buffer())
    }
}
