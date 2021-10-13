use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;

/// General XRPL Address Codec Exception.

#[derive(Debug)]
pub struct XRPLAddressCodecException(Cow<'static, str>);

impl XRPLAddressCodecException {
    pub fn new(err: &str) -> XRPLAddressCodecException {
        XRPLAddressCodecException(err.to_string().into())
    }
}

impl From<bs58::decode::Error> for XRPLAddressCodecException {
    fn from(err: bs58::decode::Error) -> Self {
        XRPLAddressCodecException(err.to_string().into())
    }
}

impl From<hex::FromHexError> for XRPLAddressCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLAddressCodecException(err.to_string().into())
    }
}

impl ToString for XRPLAddressCodecException {
    fn to_string(&self) -> String {
        self.to_owned().to_string()
    }
}
