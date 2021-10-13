use alloc::borrow::Cow;
use alloc::string::ToString;

/// General XRPL Binary Codec Exceptions.

#[derive(Debug, Clone)]
pub struct XRPLBinaryCodecException(Cow<'static, str>);

#[derive(Debug, Clone)]
pub struct VariableLengthException(Cow<'static, str>);

impl VariableLengthException {
    pub fn new(err: &str) -> VariableLengthException {
        VariableLengthException(err.to_string().into())
    }
}

impl XRPLBinaryCodecException {
    pub fn new(err: &str) -> XRPLBinaryCodecException {
        XRPLBinaryCodecException(err.to_string().into())
    }
}
