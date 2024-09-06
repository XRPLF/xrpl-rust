use reqwest::Response;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum XRPLJsonRpcException {
    #[cfg(feature = "json-rpc")]
    #[error("Reqwless error")]
    ReqwlessError,
    #[cfg(feature = "json-rpc-std")]
    #[error("Request error: {0:?}")]
    RequestError(Response),
}
