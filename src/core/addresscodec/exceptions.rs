//! General XRPL Address Codec Exception.

#[derive(Debug)]
#[non_exhaustive]
pub enum XRPLAddressCodecException {
    InvalidXAddressPrefix,
    UnsupportedXAddress,
    InvalidXAddressZeroNoTag,
    InvalidXAddressZeroRemain,
    InvalidCAddressIdLength { length: usize },
    InvalidCAddressTag,
    UnknownSeedEncoding,
    InvalidSeedEntropyLength { length: usize },
    InvalidSeedPrefixEncodingType,
    InvalidEncodingPrefixLength,
    UnexpectedPayloadLength { expected: usize, found: usize },
    Base58DecodeError(bs58::decode::Error),
    HexError(hex::FromHexError),
}

impl From<bs58::decode::Error> for XRPLAddressCodecException {
    fn from(err: bs58::decode::Error) -> Self {
        XRPLAddressCodecException::Base58DecodeError(err)
    }
}

impl From<hex::FromHexError> for XRPLAddressCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLAddressCodecException::HexError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XRPLAddressCodecException {}
