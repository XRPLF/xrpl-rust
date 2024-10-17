use alloc::string::ToString;

use crate::core::{types::Currency, BinaryParser, Parser};

use super::{exceptions::XRPLTypeException, SerializedType, TryFromParser};

pub struct Issue(SerializedType);

impl TryFromParser for Issue {
    type Error = XRPLTypeException;

    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Self, Self::Error> {
        let currency = Currency::from_parser(parser, length)?;
        let mut currency_bytes = currency.as_ref().to_vec();
        if currency.to_string() == "XRP" {
            Ok(Issue(SerializedType::from(currency_bytes)))
        } else {
            let issuer = parser.read(20)?;
            currency_bytes.extend_from_slice(&issuer);

            Ok(Issue(SerializedType::from(currency_bytes)))
        }
    }
}

impl AsRef<[u8]> for Issue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
