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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_hex_ascii() {
        let hex = str_to_hex(Cow::Borrowed("example.com")).unwrap();
        assert_eq!(hex, "6578616d706c652e636f6d");
    }

    #[test]
    fn test_str_to_hex_empty() {
        let hex = str_to_hex(Cow::Borrowed("")).unwrap();
        assert_eq!(hex, "");
    }

    #[test]
    fn test_hex_to_str_ascii() {
        let s = hex_to_str(Cow::Borrowed("6578616d706c652e636f6d")).unwrap();
        assert_eq!(s, "example.com");
    }

    #[test]
    fn test_round_trip() {
        let original = "Hello, XRPL! 🚀";
        let hex = str_to_hex(Cow::Borrowed(original)).unwrap();
        let back = hex_to_str(hex).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn test_hex_to_str_invalid_hex() {
        let result = hex_to_str(Cow::Borrowed("notvalidhex!"));
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_str_invalid_utf8() {
        // 0xff is not valid UTF-8 on its own.
        let result = hex_to_str(Cow::Borrowed("ff"));
        assert!(result.is_err());
    }
}
