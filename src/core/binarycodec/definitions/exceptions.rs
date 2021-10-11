use alloc::string::String;
use alloc::string::ToString;

/// Exception for invalid XRP Ledger definition import.
#[derive(Debug)]
pub struct XRPDefinitionException {
    message: String,
}

impl XRPDefinitionException {
    pub fn new(err: &str) -> XRPDefinitionException {
        XRPDefinitionException {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for XRPDefinitionException {
    fn from(err: serde_json::Error) -> Self {
        XRPDefinitionException {
            message: err.to_string(),
        }
    }
}
