//! Top-level exports for types used in binary_codec.

pub mod account_id;
pub mod amount;
pub mod blob;
pub mod currency;
pub mod exceptions;
pub mod hash;
pub mod paths;
pub(crate) mod test_cases;
pub mod utils;
pub mod vector256;

use crate::core::binarycodec::BinaryParser;
use alloc::vec::Vec;

/// Contains a serialized buffer of a Serializer type.
#[derive(Debug)]
pub struct SerializedType(Vec<u8>);

/// Class for serializing and deserializing Lists of objects.
///
/// See Array Fields:
/// `<https://xrpl.org/serialization.html#array-fields>`
#[derive(Debug)]
pub struct SerializedList(SerializedType);

/// Class for serializing/deserializing Indexmaps of objects.
///
/// See Object Fields:
/// `<https://xrpl.org/serialization.html#object-fields>`
#[derive(Debug)]
pub struct SerializedMap(SerializedType);

/// An XRPL Type will implement this trait.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::types::XRPLType;
/// use xrpl::core::binarycodec::exceptions::XRPLBinaryCodecException;
///
/// pub struct Example(Vec<u8>);
///
/// impl XRPLType for Example {
///     type Error = XRPLBinaryCodecException;
///
///     fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
///         Ok(Example(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
///     }
/// }
/// ```
pub trait XRPLType {
    /// Error type for implementing type.
    type Error;

    /// Create a new instance of a type.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Converter for transforming a BinaryParser into a type.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::types::TryFromParser;
/// use xrpl::core::binarycodec::BinaryParser;
/// use xrpl::core::binarycodec::Parser;
/// use xrpl::core::binarycodec::exceptions::XRPLBinaryCodecException;
///
/// pub struct Example(Vec<u8>);
///
/// impl TryFromParser for Example {
///     type Error = XRPLBinaryCodecException;
///
///     fn from_parser(
///         parser: &mut BinaryParser,
///         _length: Option<usize>,
///     ) -> Result<Example, Self::Error> {
///         Ok(Example(parser.read(42)?))
///     }
/// }
/// ```
pub trait TryFromParser {
    /// Error type for implementing type.
    type Error;

    /// Construct a type from a BinaryParser.
    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'value, T> From<T> for SerializedType
where
    T: XRPLType + AsRef<[u8]>,
{
    /// Create a serialized type from an XRPLType.
    fn from(instance: T) -> Self {
        SerializedType(instance.as_ref().to_vec())
    }
}
