use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum XRPLJsonRpcException {
    #[error("Reqwless error")]
    ReqwlessError,
    #[cfg(feature = "std")]
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
}
