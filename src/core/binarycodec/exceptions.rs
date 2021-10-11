use alloc::string::String;
use alloc::string::ToString;

/// General XRPL Binary Codec Exceptions.

#[derive(Debug, Clone)]
pub struct XRPLBinaryCodecException {
    message: String,
}

#[derive(Debug, Clone)]
pub struct VariableLengthException {
    message: String,
}

impl VariableLengthException {
    pub fn new(err: &str) -> VariableLengthException {
        VariableLengthException {
            message: err.to_string(),
        }
    }
}

impl XRPLBinaryCodecException {
    pub fn new(err: &str) -> XRPLBinaryCodecException {
        XRPLBinaryCodecException {
            message: err.to_string(),
        }
    }
}
