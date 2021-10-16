use alloc::borrow::Cow;
use alloc::string::ToString;

/// Exception for invalid XRP Ledger definition import.
#[derive(Debug)]
pub struct XRPDefinitionException(Cow<'static, str>);

impl XRPDefinitionException {
    pub fn new(err: &str) -> XRPDefinitionException {
        XRPDefinitionException(err.to_string().into())
    }
}

impl From<serde_json::Error> for XRPDefinitionException {
    fn from(err: serde_json::Error) -> Self {
        XRPDefinitionException(err.to_string().into())
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPDefinitionException {}
