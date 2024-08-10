use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum XRPLResultException {
    #[error("Response error: {0}")]
    ResponseError(String),
    #[error("Expected result or error in the response.")]
    ExpectedResultOrError,
    #[error("Unexpected result type (expected {0:?}, got {1:?}).")]
    UnexpectedResultType(String, String),
}
