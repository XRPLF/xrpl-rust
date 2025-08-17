use alloc::{borrow::Cow, string::String};

use super::exceptions::XRPLUtilsResult;

/// Convert a UTF-8-encoded string into hexadecimal encoding.
/// XRPL uses hex strings as inputs in fields like `domain`
/// in the `AccountSet` transaction.
pub fn str_to_hex<'a: 'b, 'b>(value: Cow<'a, str>) -> XRPLUtilsResult<Cow<'b, str>> {
    let hex_string = hex::encode(value.as_bytes());

    Ok(Cow::Owned(hex_string))
}

/// Convert a hex string into a human-readable string.
/// XRPL uses hex strings as inputs in fields like `domain`
/// in the `AccountSet` transaction.
pub fn hex_to_str<'a: 'b, 'b>(value: Cow<'a, str>) -> XRPLUtilsResult<Cow<'b, str>> {
    let bytes = hex::decode(value.as_ref())?;
    let string = String::from_utf8(bytes).map_err(|e| e.utf8_error())?;

    Ok(Cow::Owned(string))
}
