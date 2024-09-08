use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum XRPLJsonRpcException {
    #[error("Reqwless error")]
    ReqwlessError,
    #[cfg(feature = "std")]
    #[error("Request error: {0:?}")]
    RequestError(reqwest::Response),
}
