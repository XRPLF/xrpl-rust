//! The base class for all binary codec field types.

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Contains a serialized buffer of a Serializer type.
#[derive(Debug)]
pub struct SerializedType(Vec<u8>);

pub trait XRPLType {
    type Error;

    /// Create a new instance of a type.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub trait Buffered {
    /// Return the byte value.
    fn get_buffer(&self) -> &[u8];
}

pub trait FromParser {
    type Error;

    /// Construct a type from a BinaryParser.
    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'value, T> From<T> for SerializedType
where
    T: Buffered,
{
    fn from(instance: T) -> Self {
        SerializedType(instance.get_buffer().to_vec())
    }
}

// TODO This doesn't seem to work.
impl ToString for dyn Buffered {
    fn to_string(&self) -> String {
        hex::encode(self.get_buffer())
    }
}
