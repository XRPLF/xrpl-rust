//! General XRPL Model Exception.

use crate::models::requests::XRPLRequestException;
use crate::models::transactions::XRPLTransactionException;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Display)]
#[non_exhaustive]
pub enum XRPLModelException<'a> {
    InvalidICCannotBeXRP,
    XRPLTransactionError(XRPLTransactionException<'a>),
    XRPLRequestError(XRPLRequestException<'a>),
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
