//! Helpers for encoding and decoding XLS-89 `MPTokenMetadata`.
//!
//! On-ledger metadata is stored as a hex blob whose decoded value is a JSON
//! object. To save space the standard allows every field name to appear in a
//! short ("compact") form. These helpers convert between the human-readable
//! ("long") form developers work with and the compact form stored on-ledger,
//! encoding/decoding the hex blob along the way.
//!
//! See <https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0089-multi-purpose-token-metadata-schema>.

use alloc::string::String;

use serde::Serialize;
use serde_json::{Map, Value};

use super::exceptions::{XRPLMPTokenMetadataException, XRPLUtilsResult};

/// Maximum byte length of the on-ledger `MPTokenMetadata` blob.
pub const MAX_MPT_META_BYTE_LENGTH: usize = 1024;

/// Warning describing metadata that does not conform to the XLS-89 standard.
pub const MPT_META_WARNING_HEADER: &str = "MPTokenMetadata is not properly formatted as JSON as per the XLS-89 standard. \
While adherence to this standard is not mandatory, such non-compliant MPToken's might not be discoverable \
by Explorers and Indexers in the XRPL ecosystem.";

/// `(long, compact)` field-name pairs for the top-level metadata object.
const MPT_META_ALL_FIELDS: [(&str, &str); 9] = [
    ("ticker", "t"),
    ("name", "n"),
    ("icon", "i"),
    ("asset_class", "ac"),
    ("issuer_name", "in"),
    ("desc", "d"),
    ("asset_subclass", "as"),
    ("uris", "us"),
    ("additional_info", "ai"),
];

/// `(long, compact)` field-name pairs for each entry of the `uris` array.
const MPT_META_URI_FIELDS: [(&str, &str); 3] = [("uri", "u"), ("category", "c"), ("title", "t")];

/// Encodes `MPTokenMetadata` into the compact hex blob stored on-ledger.
///
/// The input is serialized to JSON, its known field names are shortened to
/// their compact form, the object is serialized with deterministically sorted
/// keys, and the result is hex-encoded. Unknown fields are preserved as-is, and
/// fields that already carry both the long and compact form are left untouched.
///
/// # Errors
///
/// Returns an error if `metadata` does not serialize to a JSON object.
pub fn encode_mptoken_metadata<M>(metadata: &M) -> XRPLUtilsResult<String>
where
    M: Serialize + ?Sized,
{
    let value = serde_json::to_value(metadata)?;
    let obj = value
        .as_object()
        .ok_or(XRPLMPTokenMetadataException::NotJsonObject)?;

    let mut shortened = transform_keys(obj, &MPT_META_ALL_FIELDS, Direction::Shorten);
    transform_uri_array(&mut shortened, "uris", Direction::Shorten);
    transform_uri_array(&mut shortened, "us", Direction::Shorten);

    // `serde_json` (without `preserve_order`) serializes object keys sorted,
    // giving a deterministic blob regardless of input field ordering.
    let json = serde_json::to_string(&Value::Object(shortened))?;
    Ok(hex::encode_upper(json.as_bytes()))
}

/// Decodes a hex `MPTokenMetadata` blob into a JSON object with long field names.
///
/// This is the inverse of [`encode_mptoken_metadata`]: the blob is hex-decoded,
/// parsed as JSON, and its compact field names are expanded back to their long
/// form. Unknown fields and fields carrying both forms are preserved as-is.
///
/// # Errors
///
/// Returns an error if the input is not valid hex, is not valid UTF-8, cannot be
/// parsed as JSON, or does not decode to a JSON object.
pub fn decode_mptoken_metadata(input: &str) -> XRPLUtilsResult<Value> {
    if !is_hex(input) {
        return Err(XRPLMPTokenMetadataException::InvalidHex.into());
    }

    let bytes = hex::decode(input)?;
    let text = String::from_utf8(bytes).map_err(|e| e.utf8_error())?;
    let value: Value = serde_json::from_str(&text)?;
    let obj = value
        .as_object()
        .ok_or(XRPLMPTokenMetadataException::NotJsonObject)?;

    let mut expanded = transform_keys(obj, &MPT_META_ALL_FIELDS, Direction::Expand);
    transform_uri_array(&mut expanded, "uris", Direction::Expand);
    transform_uri_array(&mut expanded, "us", Direction::Expand);

    Ok(Value::Object(expanded))
}

/// Direction of a long <-> compact field-name conversion.
#[derive(Clone, Copy)]
enum Direction {
    /// Rewrite long field names to their compact form.
    Shorten,
    /// Rewrite compact field names to their long form.
    Expand,
}

/// Rewrites the keys of `input` between long and compact form per `mappings`.
///
/// Keys not present in `mappings` are copied verbatim. When both the long and
/// compact form of a mapped field are present, both keys are kept as-is to
/// avoid silently dropping data.
fn transform_keys(
    input: &Map<String, Value>,
    mappings: &[(&str, &str)],
    direction: Direction,
) -> Map<String, Value> {
    let mut output = Map::new();

    for (key, value) in input {
        match mappings
            .iter()
            .find(|pair| pair.0 == key.as_str() || pair.1 == key.as_str())
        {
            None => {
                output.insert(key.clone(), value.clone());
            }
            Some(&(long, compact)) => {
                if input.contains_key(long) && input.contains_key(compact) {
                    output.insert(key.clone(), value.clone());
                } else {
                    let renamed = match direction {
                        Direction::Shorten => compact,
                        Direction::Expand => long,
                    };
                    output.insert(String::from(renamed), value.clone());
                }
            }
        }
    }

    output
}

/// Applies [`transform_keys`] to each object element of the array at `key`.
fn transform_uri_array(map: &mut Map<String, Value>, key: &str, direction: Direction) {
    let transformed = match map.get(key) {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|elem| match elem.as_object() {
                Some(obj) => Value::Object(transform_keys(obj, &MPT_META_URI_FIELDS, direction)),
                None => elem.clone(),
            })
            .collect::<alloc::vec::Vec<Value>>(),
        _ => return,
    };

    map.insert(String::from(key), Value::Array(transformed));
}

/// Returns `true` if `value` is a non-empty string of hexadecimal characters.
fn is_hex(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, string::String, vec::Vec};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct EncodeDecodeCase {
        #[serde(rename = "testName")]
        test_name: String,
        #[serde(rename = "mptMetadata")]
        mpt_metadata: Value,
        #[serde(rename = "expectedLongForm")]
        expected_long_form: Value,
        hex: String,
    }

    #[test]
    fn test_encode_decode_fixtures() {
        let data = include_str!("./test_data/mptoken_metadata_encode_decode.json");
        let cases: Vec<EncodeDecodeCase> = serde_json::from_str(data).unwrap();

        for case in cases {
            let encoded = encode_mptoken_metadata(&case.mpt_metadata).unwrap();
            assert_eq!(encoded, case.hex, "encode mismatch for `{}`", case.test_name);

            let decoded = decode_mptoken_metadata(&case.hex).unwrap();
            assert_eq!(
                decoded, case.expected_long_form,
                "decode mismatch for `{}`",
                case.test_name
            );
        }
    }

    #[test]
    fn test_encode_rejects_non_object() {
        let err = encode_mptoken_metadata(&Value::String("nope".into())).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!(
                "XRPL MPTokenMetadata error: {}",
                XRPLMPTokenMetadataException::NotJsonObject
            )
        );
    }

    #[test]
    fn test_decode_rejects_non_hex() {
        let err = decode_mptoken_metadata("not-hex!").unwrap_err();
        assert_eq!(
            err.to_string(),
            format!(
                "XRPL MPTokenMetadata error: {}",
                XRPLMPTokenMetadataException::InvalidHex
            )
        );
    }
}
