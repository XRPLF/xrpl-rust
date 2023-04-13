//! General XRPL Model Exception.

use crate::models::requests::XRPLRequestException;
use crate::models::transactions::XRPLTransactionException;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

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
