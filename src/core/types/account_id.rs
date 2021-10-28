//! Codec for currency property inside an XRPL
//! issued currency amount json.

use crate::constants::ACCOUNT_ID_LENGTH;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::addresscodec::*;
use crate::core::types::exceptions::XRPLHashException;
use crate::core::types::*;
use crate::core::BinaryParser;
use crate::utils::is_hex_address;
use alloc::string::String;
use alloc::string::ToString;
use core::convert::TryFrom;
use serde::ser::Error;
use serde::Serializer;
use serde::{Deserialize, Serialize};

/// Codec for serializing and deserializing AccountID fields.
///
/// See AccountID Fields:
/// `<https://xrpl.org/serialization.html#accountid-fields>`
///
///
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct AccountId(Hash160);

impl XRPLType for AccountId {
    type Error = XRPLHashException;

    /// Construct an AccountID from given bytes.
    /// If buffer is not provided, default to 20 zero bytes.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        let hash160 = Hash160::new(buffer.or(Some(&[0; ACCOUNT_ID_LENGTH])))?;
        Ok(AccountId(hash160))
    }
}

impl TryFromParser for AccountId {
    type Error = XRPLHashException;

    /// Build AccountId from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<AccountId, Self::Error> {
        Ok(AccountId(Hash160::from_parser(parser, length)?))
    }
}

impl Serialize for AccountId {
    /// Return the value of this AccountID encoded as a
    /// base58 string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let result = &encode_classic_address(self.as_ref());

        if let Ok(data) = result {
            serializer.serialize_str(data)
        } else {
            Err(S::Error::custom(result.as_ref().unwrap_err()))
        }
    }
}

impl TryFrom<&str> for AccountId {
    type Error = XRPLHashException;

    /// Construct an AccountId from a hex string or
    /// a base58 r-Address.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if is_hex_address(value) {
            Self::new(Some(&hex::decode(value)?))
        } else if is_valid_classic_address(value) {
            Self::new(Some(&decode_classic_address(value)?))
        } else if is_valid_xaddress(value) {
            let (classic_address, _, _) = xaddress_to_classic_address(value)?;
            Self::new(Some(&decode_classic_address(&classic_address)?))
        } else {
            Err(XRPLHashException::XRPLAddressCodecError(
                XRPLAddressCodecException::InvalidClassicAddressValue,
            ))
        }
    }
}

impl ToString for AccountId {
    /// Get the classic address of the AccountId bytes.
    fn to_string(&self) -> String {
        encode_classic_address(self.as_ref()).expect("to_string")
    }
}

impl AsRef<[u8]> for AccountId {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::format;

    const HEX_ENCODING: &str = "5E7B112523F68D2F5E879DB4EAC51C6698A69304";
    const BASE58_ENCODING: &str = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59";

    #[test]
    fn test_accountid_new() {
        let hex = hex::decode(HEX_ENCODING).expect("");
        let account = AccountId::new(Some(&hex)).unwrap();
        assert_eq!(HEX_ENCODING, hex::encode_upper(account));
    }

    #[test]
    fn test_accountid_try_from() {
        let account = AccountId::try_from(BASE58_ENCODING).unwrap();
        assert_eq!(HEX_ENCODING, hex::encode_upper(account));
    }

    #[test]
    fn accept_accountid_serde_encode_decode() {
        let account: AccountId = AccountId::try_from(BASE58_ENCODING).unwrap();
        let serialize = serde_json::to_string(&account).unwrap();
        let deserialize: AccountId = serde_json::from_str(&serialize).unwrap();

        assert_eq!(format!("\"{}\"", BASE58_ENCODING), serialize);
        assert_eq!(account.to_string(), deserialize.to_string());
    }
}
