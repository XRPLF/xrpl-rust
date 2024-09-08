//! General XRPL Model Exception.

use alloc::string::String;
use serde::{Deserialize, Serialize};
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLModelException<'a> {
    #[error("Missing Field: {0}")]
    MissingField(&'a str),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct JSONRPCException {
    code: i32,
    message: String,
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLModelException<'a> {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Error)]
pub enum XRPLFlagsException {
    #[error("Cannot convert flag to u32")]
    CannotConvertFlagToU32,
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLFlagsException {}
