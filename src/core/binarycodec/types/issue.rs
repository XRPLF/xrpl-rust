use alloc::string::ToString;
use alloc::vec::Vec;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::core::{
    binarycodec::types::{AccountId, Currency},
    exceptions::{XRPLCoreException, XRPLCoreResult},
    BinaryParser, Parser,
};

use super::{exceptions::XRPLTypeException, SerializedType, TryFromParser, XRPLType};

/// Width of an MPT Issue in bytes: issuer(20) + NO_ACCOUNT(20) + sequence(4) = 44
const MPT_WIDTH: usize = 44;

/// Sentinel account ID used to distinguish MPT issues from IOU issues.
const NO_ACCOUNT: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

#[derive(Debug, Clone)]
pub struct Issue(SerializedType);

impl XRPLType for Issue {
    type Error = XRPLCoreException;

    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Issue(SerializedType(buffer.unwrap_or(&[]).to_vec())))
    }
}

impl TryFromParser for Issue {
    type Error = XRPLCoreException;

    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> XRPLCoreResult<Self, Self::Error> {
        // Read first 20 bytes (currency or issuer account for MPT)
        let currency = Currency::from_parser(parser, length)?;
        let mut bytes = currency.as_ref().to_vec();

        if currency.to_string() == "XRP" {
            // XRP issue: just 20 bytes of zero
            Ok(Issue(SerializedType::from(bytes)))
        } else {
            // Read next 20 bytes (issuer for IOU, or NO_ACCOUNT sentinel for MPT)
            let next_20 = parser.read(20)?;
            if next_20 == NO_ACCOUNT {
                // MPT: read 4 more bytes for sequence
                let sequence = parser.read(4)?;
                bytes.extend_from_slice(&next_20);
                bytes.extend_from_slice(&sequence);
                Ok(Issue(SerializedType::from(bytes)))
            } else {
                // IOU: currency + issuer
                bytes.extend_from_slice(&next_20);
                Ok(Issue(SerializedType::from(bytes)))
            }
        }
    }
}

impl TryFrom<Value> for Issue {
    type Error = XRPLCoreException;

    fn try_from(value: Value) -> XRPLCoreResult<Self, Self::Error> {
        if value.get("currency") == Some(&Value::String("XRP".to_string())) {
            let currency = Currency::try_from("XRP")?;
            Ok(Issue(SerializedType::from(currency.as_ref().to_vec())))
        } else if let Some(obj) = value.as_object() {
            if let Some(mpt_id_val) = obj.get("mpt_issuance_id") {
                // MPT Issue
                let mpt_id = mpt_id_val.as_str().ok_or(XRPLTypeException::MissingField(
                    "mpt_issuance_id".to_string(),
                ))?;
                let mpt_bytes = hex::decode(mpt_id).map_err(|_| {
                    XRPLCoreException::XRPLUtilsError("Invalid mpt_issuance_id hex".to_string())
                })?;
                if mpt_bytes.len() != 24 {
                    return Err(XRPLCoreException::XRPLUtilsError(
                        "mpt_issuance_id has invalid hash length".to_string(),
                    ));
                }
                // mpt_issuance_id: first 4 bytes = sequence (big-endian), last 20 bytes = issuer
                let sequence_be = &mpt_bytes[0..4];
                let issuer_account = &mpt_bytes[4..24];

                // Convert sequence to little-endian
                let sequence = u32::from_be_bytes(sequence_be.try_into().map_err(|_| {
                    XRPLCoreException::XRPLUtilsError("Invalid sequence bytes".to_string())
                })?);
                let sequence_le = sequence.to_le_bytes();

                // Build: issuer(20) + NO_ACCOUNT(20) + sequence_le(4) = 44 bytes
                let mut result: Vec<u8> = Vec::with_capacity(MPT_WIDTH);
                result.extend_from_slice(issuer_account);
                result.extend_from_slice(&NO_ACCOUNT);
                result.extend_from_slice(&sequence_le);

                Ok(Issue(SerializedType::from(result)))
            } else {
                // IOU Issue
                let cur = obj
                    .get("currency")
                    .and_then(|v| v.as_str())
                    .ok_or(XRPLTypeException::MissingField("currency".to_string()))?;
                let currency = Currency::try_from(cur)?;
                let issuer = obj
                    .get("issuer")
                    .and_then(|v| v.as_str())
                    .ok_or(XRPLTypeException::MissingField("issuer".to_string()))?;
                let account = AccountId::try_from(issuer)?;
                let mut currency_bytes = currency.as_ref().to_vec();
                currency_bytes.extend_from_slice(account.as_ref());
                Ok(Issue(SerializedType::from(currency_bytes)))
            }
        } else {
            Err(XRPLTypeException::UnexpectedJSONType.into())
        }
    }
}

impl Serialize for Issue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_ref();

        // If exactly 44 bytes, this is an MPT Issue
        if bytes.len() == MPT_WIDTH {
            let issuer_account = &bytes[0..20];
            // bytes[20..40] = NO_ACCOUNT (skip)
            let sequence_le = &bytes[40..44];
            let sequence = u32::from_le_bytes(
                sequence_le
                    .try_into()
                    .map_err(|e: core::array::TryFromSliceError| serde::ser::Error::custom(e))?,
            );

            // Reconstruct mpt_issuance_id: sequence(BE, 4) + issuer(20) = 24 bytes
            let sequence_be = sequence.to_be_bytes();
            let mut mpt_id = Vec::with_capacity(24);
            mpt_id.extend_from_slice(&sequence_be);
            mpt_id.extend_from_slice(issuer_account);

            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("mpt_issuance_id", &hex::encode_upper(&mpt_id))?;
            map.end()
        } else {
            // Parse the currency from the first 20 bytes
            let mut parser = BinaryParser::from(bytes);
            let currency =
                Currency::from_parser(&mut parser, None).map_err(serde::ser::Error::custom)?;

            if currency.to_string() == "XRP" {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("currency", "XRP")?;
                map.end()
            } else {
                // Next 20 bytes are the issuer
                let issuer =
                    AccountId::from_parser(&mut parser, None).map_err(serde::ser::Error::custom)?;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("currency", &currency)?;
                map.serialize_entry("issuer", &issuer)?;
                map.end()
            }
        }
    }
}

impl AsRef<[u8]> for Issue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
