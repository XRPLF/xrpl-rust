use strum_macros::Display;
use thiserror_no_std::Error;

#[derive(Debug, Error, Display)]
pub enum XRPLJsonRpcException {
    ReqwlessError(reqwless::Error),
}
