use alloc::string::String;

use thiserror_no_std::Error;

use crate::asynch::clients::exceptions::XRPLNetworkErrorKind;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XRPLJsonRpcException {
    #[error("Reqwless error: {0:?}")]
    ReqwlessError(#[from] reqwless::Error),
    #[cfg(feature = "std")]
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Request error: {0}")]
    RequestError(String),
}

impl XRPLJsonRpcException {
    /// Return a typed network error category, if this JSON-RPC failure is transport-related.
    pub fn network_error_kind(&self) -> Option<XRPLNetworkErrorKind> {
        match self {
            #[cfg(feature = "std")]
            XRPLJsonRpcException::ReqwestError(error) => {
                if error.is_timeout() {
                    Some(XRPLNetworkErrorKind::TimedOut)
                } else if error.is_connect() {
                    Some(XRPLNetworkErrorKind::OtherIo)
                } else {
                    None
                }
            }
            XRPLJsonRpcException::ReqwlessError(_) => Some(XRPLNetworkErrorKind::OtherIo),
            XRPLJsonRpcException::RequestError(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_errors_are_not_network_errors() {
        let error = XRPLJsonRpcException::RequestError("bad request".into());
        assert_eq!(error.network_error_kind(), None);
    }
}
