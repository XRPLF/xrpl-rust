use alloc::borrow::Cow;
use alloc::fmt::Display;
use alloc::fmt::Formatter;
use alloc::string::ToString;

/// Exception for invalid XRP Ledger time data.
#[derive(Debug)]
pub struct XRPRangeException(Cow<'static, str>);

impl XRPRangeException {
    pub fn new(err: &str) -> XRPRangeException {
        XRPRangeException(err.to_string().into())
    }
}

impl Display for XRPRangeException {
    fn fmt(&self, f: &mut Formatter) -> alloc::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<rust_decimal::Error> for XRPRangeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPRangeException(err.to_string().into())
    }
}
