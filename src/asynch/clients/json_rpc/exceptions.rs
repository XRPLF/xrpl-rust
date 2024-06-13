use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum XRPLJsonRpcException {
    #[cfg(feature = "json-rpc")]
    #[error("Reqwless error")]
    ReqwlessError,
}
