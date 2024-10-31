use thiserror_no_std::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XRPLJsonRpcException {
    #[error("Reqwless error: {0:?}")]
    ReqwlessError(#[from] reqwless::Error),
    #[cfg(feature = "std")]
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
}
