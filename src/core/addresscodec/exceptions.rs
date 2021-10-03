/// General XRPL Address Codec Exception.

#[derive(Debug)]
pub struct XRPLAddressCodecException {
    message: String,
}

impl XRPLAddressCodecException {
    pub fn new(err: &str) -> XRPLAddressCodecException {
        XRPLAddressCodecException {
            message: err.to_string(),
        }
    }
}

impl From<bs58::decode::Error> for XRPLAddressCodecException {
    fn from(err: bs58::decode::Error) -> Self {
        XRPLAddressCodecException {
            message: err.to_string(),
        }
    }
}

impl From<hex::FromHexError> for XRPLAddressCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLAddressCodecException {
            message: err.to_string(),
        }
    }
}
