//! Core codec functions for interacting with the XRPL.

pub mod addresscodec;
pub mod binarycodec;
pub mod definitions;
pub mod keypairs;
pub mod types;

pub use self::binarycodec::binary_wrappers::BinaryParser;
pub use self::binarycodec::binary_wrappers::BinarySerializer;
pub use self::binarycodec::binary_wrappers::Parser;
pub use self::definitions::load_definition_map;
