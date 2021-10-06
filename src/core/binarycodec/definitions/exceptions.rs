#[derive(Debug)]
/// Exception for invalid XRP Ledger definition import.
pub struct XRPDefinitionException {
    message: String,
}

#[cfg(feature = "std")]
impl std::convert::From<serde_json::Error> for XRPDefinitionException {
    fn from(err: serde_json::Error) -> Self {
        XRPDefinitionException {
            message: err.to_string(),
        }
    }
}
