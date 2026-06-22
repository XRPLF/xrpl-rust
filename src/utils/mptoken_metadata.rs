//! Helpers for encoding and decoding XLS-89 `MPTokenMetadata`.
//!
//! On-ledger metadata is stored as a hex blob whose decoded value is a JSON
//! object. To save space the standard allows every field name to appear in a
//! short ("compact") form. These helpers convert between the human-readable
//! ("long") form developers work with and the compact form stored on-ledger,
//! encoding/decoding the hex blob along the way.
//!
//! See <https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0089-multi-purpose-token-metadata-schema>.

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::exceptions::{XRPLMPTokenMetadataException, XRPLUtilsResult};

/// Maximum byte length of the on-ledger `MPTokenMetadata` blob.
pub const MAX_MPT_META_BYTE_LENGTH: usize = 1024;

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

/// Allowed values for the `asset_class` field.
const MPT_META_ASSET_CLASSES: [&str; 6] = ["rwa", "memes", "wrapped", "gaming", "defi", "other"];

/// Allowed values for the `asset_subclass` field.
const MPT_META_ASSET_SUB_CLASSES: [&str; 7] = [
    "stablecoin",
    "commodity",
    "real_estate",
    "private_credit",
    "equity",
    "treasury",
    "other",
];

/// A related URI entry within [`MPTokenMetadata`], per XLS-89.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenMetadataUri<'a> {
    /// URI to the related resource (e.g. `https://...` or `ipfs://...`).
    pub uri: Cow<'a, str>,
    /// Category of the link: `website`, `social`, `docs`, or `other`.
    pub category: Cow<'a, str>,
    /// Human-readable label for the link.
    pub title: Cow<'a, str>,
}

/// The freeform `additional_info` field: either UTF-8 text or a JSON object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MPTokenMetadataAdditionalInfo<'a> {
    /// A UTF-8 string.
    Text(Cow<'a, str>),
    /// An arbitrary JSON object.
    Object(Map<String, Value>),
}

/// `MPTokenMetadata` as described by the XLS-89 standard.
///
/// Build this in long form and pass it to [`encode_mptoken_metadata`] to
/// produce the compact hex blob stored on-ledger;
/// [`decode_mptoken_metadata`] produces a JSON value that deserializes back
/// into this type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenMetadata<'a> {
    /// Ticker symbol: uppercase letters (A-Z) and digits (0-9), max 6 chars.
    pub ticker: Cow<'a, str>,
    /// Display name of the token.
    pub name: Cow<'a, str>,
    /// Short description of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<Cow<'a, str>>,
    /// URI to the token icon.
    pub icon: Cow<'a, str>,
    /// Top-level classification: `rwa`, `memes`, `wrapped`, `gaming`, `defi`, or `other`.
    pub asset_class: Cow<'a, str>,
    /// Subcategory of the asset class. Required when `asset_class` is `rwa`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_subclass: Option<Cow<'a, str>>,
    /// Name of the issuer account.
    pub issuer_name: Cow<'a, str>,
    /// Related URIs (site, dashboard, social media, documentation, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uris: Option<Vec<MPTokenMetadataUri<'a>>>,
    /// Freeform field for key token details (interest rate, maturity, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<MPTokenMetadataAdditionalInfo<'a>>,
}

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

/// Returns `true` if `value` is a non-empty, even-length string of hexadecimal
/// characters. Hex byte strings always have an even number of characters.
fn is_hex(value: &str) -> bool {
    !value.is_empty() && value.len() % 2 == 0 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Validates that a hex `MPTokenMetadata` blob adheres to the XLS-89 standard.
///
/// Returns a list of human-readable messages describing every way in which the
/// blob fails to conform. An empty list means the metadata is valid. Both the
/// long and compact form of each field are accepted, and unknown fields are
/// allowed (subject to the top-level field-count limit).
pub fn validate_mptoken_metadata(input: &str) -> Vec<String> {
    let mut messages = Vec::new();

    if !is_hex(input) {
        messages.push("MPTokenMetadata must be in hex format.".to_string());
        return messages;
    }

    if input.len() / 2 > MAX_MPT_META_BYTE_LENGTH {
        messages.push(format!(
            "MPTokenMetadata must be max {MAX_MPT_META_BYTE_LENGTH} bytes."
        ));
        return messages;
    }

    // `is_hex` guarantees an even-length hex string, so decoding cannot fail; the
    // decoded bytes may still be invalid UTF-8, which is reported below.
    let bytes = hex::decode(input).unwrap_or_default();
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) => {
            messages.push(format!(
                "MPTokenMetadata is not properly formatted as JSON - {err}"
            ));
            return messages;
        }
    };

    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            messages.push(format!(
                "MPTokenMetadata is not properly formatted as JSON - {err}"
            ));
            return messages;
        }
    };

    let obj = match value.as_object() {
        Some(obj) => obj,
        None => {
            messages.push(
                "MPTokenMetadata is not properly formatted JSON object as per XLS-89.".to_string(),
            );
            return messages;
        }
    };

    if obj.len() > MPT_META_ALL_FIELDS.len() {
        messages.push(format!(
            "MPTokenMetadata must not contain more than {} top-level fields (found {}).",
            MPT_META_ALL_FIELDS.len(),
            obj.len()
        ));
    }

    // Field order is significant: messages are emitted in this order.
    messages.extend(validate_ticker(obj));
    messages.extend(validate_non_empty_string(obj, "name", "n"));
    messages.extend(validate_non_empty_string(obj, "icon", "i"));
    messages.extend(validate_asset_class(obj));
    messages.extend(validate_non_empty_string(obj, "issuer_name", "in"));
    messages.extend(validate_optional_non_empty_string(obj, "desc", "d"));
    messages.extend(validate_asset_subclass(obj));
    messages.extend(validate_uris(obj));
    messages.extend(validate_additional_info(obj));

    messages
}

/// JS `obj[key] != null`: the key is present and not JSON null.
fn present_non_null(obj: &Map<String, Value>, key: &str) -> bool {
    matches!(obj.get(key), Some(value) if !value.is_null())
}

/// JS `obj[long] != null && obj[compact] != null`.
fn has_both_forms(obj: &Map<String, Value>, long: &str, compact: &str) -> bool {
    present_non_null(obj, long) && present_non_null(obj, compact)
}

/// JS `obj[long] === undefined && obj[compact] === undefined`.
fn neither_form_present(obj: &Map<String, Value>, long: &str, compact: &str) -> bool {
    obj.get(long).is_none() && obj.get(compact).is_none()
}

/// JS `obj[long] ?? obj[compact]`: prefer the long form unless null/absent.
fn coalesce<'a>(obj: &'a Map<String, Value>, long: &str, compact: &str) -> Option<&'a Value> {
    match obj.get(long) {
        Some(value) if !value.is_null() => Some(value),
        _ => obj.get(compact),
    }
}

fn is_string(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::String(_)))
}

fn is_non_empty_string(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::String(s)) if !s.is_empty())
}

fn equals_str(value: Option<&Value>, expected: &str) -> bool {
    matches!(value, Some(Value::String(s)) if s == expected)
}

fn both_forms_message(long: &str, compact: &str) -> String {
    format!("{long}/{compact}: both long and compact forms present. expected only one.")
}

fn validate_ticker(obj: &Map<String, Value>) -> Vec<String> {
    if has_both_forms(obj, "ticker", "t") {
        return vec![both_forms_message("ticker", "t")];
    }
    let valid =
        matches!(coalesce(obj, "ticker", "t"), Some(Value::String(s)) if is_valid_ticker(s));
    if !valid {
        return vec![
            "ticker/t: should have uppercase letters (A-Z) and digits (0-9) only. Max 6 characters recommended."
                .to_string(),
        ];
    }
    Vec::new()
}

fn is_valid_ticker(value: &str) -> bool {
    let len = value.chars().count();
    (1..=6).contains(&len)
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn validate_non_empty_string(obj: &Map<String, Value>, long: &str, compact: &str) -> Vec<String> {
    if has_both_forms(obj, long, compact) {
        return vec![both_forms_message(long, compact)];
    }
    if !is_non_empty_string(coalesce(obj, long, compact)) {
        return vec![format!("{long}/{compact}: should be a non-empty string.")];
    }
    Vec::new()
}

fn validate_optional_non_empty_string(
    obj: &Map<String, Value>,
    long: &str,
    compact: &str,
) -> Vec<String> {
    if has_both_forms(obj, long, compact) {
        return vec![both_forms_message(long, compact)];
    }
    if neither_form_present(obj, long, compact) {
        return Vec::new();
    }
    if !is_non_empty_string(coalesce(obj, long, compact)) {
        return vec![format!("{long}/{compact}: should be a non-empty string.")];
    }
    Vec::new()
}

fn validate_asset_class(obj: &Map<String, Value>) -> Vec<String> {
    if has_both_forms(obj, "asset_class", "ac") {
        return vec![both_forms_message("asset_class", "ac")];
    }
    let value = coalesce(obj, "asset_class", "ac");
    let valid =
        matches!(value, Some(Value::String(s)) if MPT_META_ASSET_CLASSES.contains(&s.as_str()));
    if !valid {
        return vec![format!(
            "asset_class/ac: should be one of {}.",
            MPT_META_ASSET_CLASSES.join(", ")
        )];
    }
    Vec::new()
}

fn validate_asset_subclass(obj: &Map<String, Value>) -> Vec<String> {
    if has_both_forms(obj, "asset_subclass", "as") {
        return vec![both_forms_message("asset_subclass", "as")];
    }
    let value = coalesce(obj, "asset_subclass", "as");
    let is_rwa = equals_str(obj.get("asset_class"), "rwa") || equals_str(obj.get("ac"), "rwa");
    if is_rwa && value.is_none() {
        return vec!["asset_subclass/as: required when asset_class is rwa.".to_string()];
    }
    if neither_form_present(obj, "asset_subclass", "as") {
        return Vec::new();
    }
    let valid =
        matches!(value, Some(Value::String(s)) if MPT_META_ASSET_SUB_CLASSES.contains(&s.as_str()));
    if !valid {
        return vec![format!(
            "asset_subclass/as: should be one of {}.",
            MPT_META_ASSET_SUB_CLASSES.join(", ")
        )];
    }
    Vec::new()
}

fn validate_uris(obj: &Map<String, Value>) -> Vec<String> {
    if has_both_forms(obj, "uris", "us") {
        return vec![both_forms_message("uris", "us")];
    }
    if neither_form_present(obj, "uris", "us") {
        return Vec::new();
    }

    let arr = match coalesce(obj, "uris", "us") {
        Some(Value::Array(arr)) if !arr.is_empty() => arr,
        _ => return vec!["uris/us: should be a non-empty array.".to_string()],
    };

    let structure_message =
        "uris/us: should be an array of objects each with uri/u, category/c, and title/t properties.";
    let mut messages = Vec::new();

    for elem in arr {
        let uri_obj = match elem.as_object() {
            Some(uri_obj) if uri_obj.len() == MPT_META_URI_FIELDS.len() => uri_obj,
            _ => {
                messages.push(structure_message.to_string());
                continue;
            }
        };

        for &(long, compact) in &MPT_META_URI_FIELDS {
            if has_both_forms(uri_obj, long, compact) {
                messages.push(format!(
                    "uris/us: should not have both {long} and {compact} fields."
                ));
                break;
            }
        }

        let uri = coalesce(uri_obj, "uri", "u");
        let category = coalesce(uri_obj, "category", "c");
        let title = coalesce(uri_obj, "title", "t");
        if !(is_string(uri) && is_string(category) && is_string(title)) {
            messages.push(structure_message.to_string());
        }
    }

    messages
}

fn validate_additional_info(obj: &Map<String, Value>) -> Vec<String> {
    if has_both_forms(obj, "additional_info", "ai") {
        return vec![both_forms_message("additional_info", "ai")];
    }
    if neither_form_present(obj, "additional_info", "ai") {
        return Vec::new();
    }
    let value = coalesce(obj, "additional_info", "ai");
    if !matches!(value, Some(Value::String(_)) | Some(Value::Object(_))) {
        return vec!["additional_info/ai: should be a string or JSON object.".to_string()];
    }
    Vec::new()
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
            assert_eq!(
                encoded, case.hex,
                "encode mismatch for `{}`",
                case.test_name
            );

            let decoded = decode_mptoken_metadata(&case.hex).unwrap();
            assert_eq!(
                decoded, case.expected_long_form,
                "decode mismatch for `{}`",
                case.test_name
            );
        }
    }

    #[derive(Deserialize)]
    struct ValidationCase {
        #[serde(rename = "testName")]
        test_name: String,
        #[serde(rename = "mptMetadata")]
        mpt_metadata: Value,
        #[serde(rename = "validationMessages")]
        validation_messages: Vec<String>,
    }

    #[test]
    fn test_validation_fixtures() {
        // serde_json's parse-error text differs from V8's, so messages on this
        // path are matched by prefix only.
        const JSON_PARSE_PREFIX: &str = "MPTokenMetadata is not properly formatted as JSON -";

        let data = include_str!("./test_data/mptoken_metadata_validation.json");
        let cases: Vec<ValidationCase> = serde_json::from_str(data).unwrap();

        for case in cases {
            // The fixture harness uses a raw string value as the blob payload
            // directly; any other value is serialized to JSON first.
            let payload = match &case.mpt_metadata {
                Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap(),
            };
            let hex = hex::encode_upper(payload.as_bytes());

            let actual = validate_mptoken_metadata(&hex);
            assert_eq!(
                actual.len(),
                case.validation_messages.len(),
                "message count mismatch for `{}`: {actual:?}",
                case.test_name
            );

            for (got, want) in actual.iter().zip(case.validation_messages.iter()) {
                if want.starts_with(JSON_PARSE_PREFIX) {
                    assert!(
                        got.starts_with(JSON_PARSE_PREFIX),
                        "expected JSON-parse message for `{}`, got: {got}",
                        case.test_name
                    );
                } else {
                    assert_eq!(got, want, "message mismatch for `{}`", case.test_name);
                }
            }
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

    #[test]
    fn test_typed_metadata_round_trip() {
        let metadata = MPTokenMetadata {
            ticker: "TBILL".into(),
            name: "T-Bill Yield Token".into(),
            desc: Some("A yield-bearing stablecoin backed by U.S. Treasuries.".into()),
            icon: "https://example.org/tbill-icon.png".into(),
            asset_class: "rwa".into(),
            asset_subclass: Some("treasury".into()),
            issuer_name: "Example Yield Co.".into(),
            uris: Some(vec![MPTokenMetadataUri {
                uri: "https://exampleyield.co/tbill".into(),
                category: "website".into(),
                title: "Product Page".into(),
            }]),
            additional_info: Some(MPTokenMetadataAdditionalInfo::Object(
                serde_json::json!({ "interest_rate": "5.00%", "maturity_date": "2045-06-30" })
                    .as_object()
                    .unwrap()
                    .clone(),
            )),
        };

        let encoded = encode_mptoken_metadata(&metadata).unwrap();
        assert!(validate_mptoken_metadata(&encoded).is_empty());

        let decoded = decode_mptoken_metadata(&encoded).unwrap();
        let round_trip: MPTokenMetadata = serde_json::from_value(decoded).unwrap();
        assert_eq!(round_trip, metadata);
    }

    /// Hex-encode a JSON value the way the ledger stores it, then validate it.
    fn validate_json(value: serde_json::Value) -> Vec<String> {
        let hex = hex::encode_upper(serde_json::to_string(&value).unwrap().as_bytes());
        validate_mptoken_metadata(&hex)
    }

    #[test]
    fn test_validate_rejects_non_hex_input() {
        let expected = vec!["MPTokenMetadata must be in hex format.".to_string()];
        // Non-hex characters.
        assert_eq!(validate_mptoken_metadata("xyz"), expected);
        // Valid hex characters but an odd length is not a valid hex byte string.
        assert_eq!(validate_mptoken_metadata("ABC"), expected);
    }

    #[test]
    fn test_validate_reports_non_utf8_blob() {
        // Even-length valid hex that decodes to 0xFF, which is not valid UTF-8.
        let messages = validate_mptoken_metadata("FF");
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("MPTokenMetadata is not properly formatted as JSON -"));
    }

    #[test]
    fn test_validate_reports_both_forms_for_every_field() {
        use serde_json::json;

        let base = || {
            json!({
                "ticker": "TBILL",
                "name": "T-Bill",
                "icon": "https://example.org/icon.png",
                "asset_class": "rwa",
                "asset_subclass": "treasury",
                "issuer_name": "Issuer"
            })
        };
        assert!(validate_json(base()).is_empty(), "baseline should be valid");

        // Adding a single compact-form key conflicts with its long form.
        let single_key_cases = [
            ("n", json!("dup"), "name/n"),
            ("i", json!("https://dup"), "icon/i"),
            ("in", json!("dup"), "issuer_name/in"),
            ("ac", json!("rwa"), "asset_class/ac"),
            ("as", json!("treasury"), "asset_subclass/as"),
        ];
        for (key, value, prefix) in single_key_cases {
            let mut obj = base();
            obj[key] = value;
            assert_eq!(
                validate_json(obj),
                vec![format!(
                    "{prefix}: both long and compact forms present. expected only one."
                )],
                "field {prefix}"
            );
        }

        // desc / uris / additional_info are optional, so set both forms explicitly.
        let mut desc = base();
        desc["desc"] = json!("a");
        desc["d"] = json!("b");
        assert_eq!(
            validate_json(desc),
            vec!["desc/d: both long and compact forms present. expected only one.".to_string()]
        );

        let mut uris = base();
        uris["uris"] = json!([{ "uri": "https://x", "category": "website", "title": "T" }]);
        uris["us"] = json!([{ "u": "https://x", "c": "website", "t": "T" }]);
        assert_eq!(
            validate_json(uris),
            vec!["uris/us: both long and compact forms present. expected only one.".to_string()]
        );

        let mut info = base();
        info["additional_info"] = json!("x");
        info["ai"] = json!("y");
        assert_eq!(
            validate_json(info),
            vec![
                "additional_info/ai: both long and compact forms present. expected only one."
                    .to_string()
            ]
        );
    }

    #[test]
    fn test_encode_decode_preserves_non_object_uri_elements() {
        use serde_json::json;

        // Non-object entries in a uris/us array pass through encode/decode unchanged.
        let value = json!({ "ticker": "TBILL", "uris": [123, "not-an-object"] });
        let encoded = encode_mptoken_metadata(&value).unwrap();
        let decoded = decode_mptoken_metadata(&encoded).unwrap();
        assert_eq!(
            decoded,
            json!({ "ticker": "TBILL", "uris": [123, "not-an-object"] })
        );
    }
}
