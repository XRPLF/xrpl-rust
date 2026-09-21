use alloc::borrow::Cow;
use alloc::string::String;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::models::Model;

use super::{CommonFields, Request, RequestMethod};

/// A request wrapper for any rippled command that does not yet have a typed
/// [`XRPLRequest`](super::XRPLRequest) variant.
///
/// This model is xrpl-rust-specific — there is no rippled RPC named `generic`;
/// it is a client-side escape hatch modeled on
/// [`xrpl-py`'s `GenericRequest`](https://github.com/XRPLF/xrpl-py/blob/main/xrpl/models/requests/generic_request.py).
///
/// Use it for:
/// - rippled admin commands that are unlikely to gain a typed model
///   (`ledger_accept`, `stop`, `sign_for`, ...),
/// - newly-added RPCs that the library has not caught up with yet,
/// - one-off calls where the extra typing of a full [`Request`] impl is
///   overkill.
///
/// The `command` field is written verbatim into the JSON payload; any extra
/// fields in `params` are flattened alongside it. `id`, if any, is picked up
/// from [`CommonFields`] like every other request (so
/// [`XRPLClient::set_request_id`](crate::asynch::clients) still auto-fills it).
///
/// # Example
///
/// ```no_run
/// # #[cfg(all(feature = "std", feature = "json-rpc"))]
/// # async fn demo() -> anyhow::Result<()> {
/// use serde_json::Map;
/// use xrpl::asynch::clients::{AsyncJsonRpcClient, XRPLAsyncClient};
/// use xrpl::models::requests::generic_request::GenericRequest;
/// use url::Url;
///
/// let client = AsyncJsonRpcClient::connect(Url::parse("http://127.0.0.1:5005")?);
/// let request = GenericRequest::new("ledger_accept", None, Map::new());
/// let response = client.request(request.into()).await?;
/// # let _ = response;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenericRequest<'a> {
    /// The fields shared by every request. Only `id` is meaningful for
    /// `GenericRequest`; the `command` slot carries the sentinel
    /// [`RequestMethod::Generic`] and is never serialized (the real command
    /// string, from `command` below, is what goes on the wire).
    pub common_fields: CommonFields<'a>,

    /// The rippled RPC command name (e.g. `"ledger_accept"`, `"server_definitions"`).
    pub command: Cow<'a, str>,

    /// Extra fields, serialized flat next to `command` and `id`.
    ///
    /// For nested params (JSON-RPC callers wrapping in a `params: [{...}]`
    /// array), the surrounding client layer already handles that; put the
    /// inner object's keys here directly.
    pub params: Map<String, Value>,
}

impl<'a> GenericRequest<'a> {
    /// Build a `GenericRequest` for the given rippled `command`, with an
    /// optional client-supplied `id` and any extra `params`.
    pub fn new(
        command: impl Into<Cow<'a, str>>,
        id: Option<Cow<'a, str>>,
        params: Map<String, Value>,
    ) -> Self {
        // Reserved keys `command` and `id` would collide with the fields the
        // serializer emits before `params`, letting a caller-supplied
        // `command` silently override the intended RPC method. We reject them
        // at construction so the mistake is loud in dev; `serialize` also
        // filters them defensively for release builds.
        debug_assert!(
            !params.contains_key("command") && !params.contains_key("id"),
            "GenericRequest params must not contain reserved keys `command` or `id`"
        );
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Generic,
                id,
            },
            command: command.into(),
            params,
        }
    }
}

impl<'a> Model for GenericRequest<'a> {}

impl<'a> Request<'a> for GenericRequest<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Serialize for GenericRequest<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Emit: {"command": <self.command>, ["id": <id>,] ...self.params}
        // We deliberately skip `common_fields.command` (the sentinel
        // `RequestMethod::Generic`) so the real command string wins.
        //
        // `params` entries keyed `command` or `id` are dropped: they would
        // collide with the fields we emit above, and `serde_json`'s Map
        // (BTreeMap-backed) resolves duplicate keys with last-write-wins —
        // so a stray `params.insert("command", "stop")` would silently
        // redirect the RPC. `new()` catches this in debug builds; this
        // filter is defense in depth for release.
        let reserved = |k: &str| k == "command" || k == "id";
        let params_count = self.params.keys().filter(|k| !reserved(k)).count();
        let extra = 1 + usize::from(self.common_fields.id.is_some());
        let mut map = serializer.serialize_map(Some(params_count + extra))?;
        map.serialize_entry("command", &self.command)?;
        if let Some(id) = &self.common_fields.id {
            map.serialize_entry("id", id)?;
        }
        for (k, v) in &self.params {
            if !reserved(k) {
                map.serialize_entry(k, v)?;
            }
        }
        map.end()
    }
}

// Deserialize is implemented WITHOUT an explicit `'de: 'a` bound because our
// visitor materializes owned `String`s into `Cow::Owned` — there is no borrow
// from the deserializer's input. `GenericRequest<'a>` is covariant in `'a`
// (all lifetime uses are behind `Cow<'a, str>`, which is covariant), so the
// owned `GenericRequest<'static>` returned by the visitor coerces to any
// caller-requested `'a`. Keeping the bound off means the derived
// `Deserialize` on `XRPLRequest<'a>` does not pick up an extra `'de: 'a`
// requirement it cannot satisfy.
impl<'de, 'a> Deserialize<'de> for GenericRequest<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GenericRequestVisitor;

        impl<'de> Visitor<'de> for GenericRequestVisitor {
            type Value = GenericRequest<'static>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("a JSON object with at least a `command` string field")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut command: Option<String> = None;
                let mut id: Option<String> = None;
                let mut params: Map<String, Value> = Map::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "command" => {
                            if command.is_some() {
                                return Err(de::Error::duplicate_field("command"));
                            }
                            command = Some(map.next_value()?);
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        _ => {
                            let value: Value = map.next_value()?;
                            params.insert(key, value);
                        }
                    }
                }

                let command = command.ok_or_else(|| de::Error::missing_field("command"))?;
                Ok(GenericRequest::new(
                    Cow::Owned(command),
                    id.map(Cow::Owned),
                    params,
                ))
            }
        }

        // `GenericRequest<'a>` is covariant in `'a`, so this coerces from
        // `GenericRequest<'static>` — no `'de: 'a` bound needed.
        deserializer.deserialize_map(GenericRequestVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use serde_json::json;

    #[test]
    fn test_serialize_no_params_no_id() {
        let req = GenericRequest::new("ledger_accept", None, Map::new());
        let value: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(value, json!({"command": "ledger_accept"}));
    }

    #[test]
    fn test_serialize_with_id() {
        let req = GenericRequest::new("ledger_accept", Some("req-1".into()), Map::new());
        let value: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(value, json!({"command": "ledger_accept", "id": "req-1"}));
    }

    #[test]
    fn test_serialize_with_params() {
        let mut params = Map::new();
        params.insert("ledger_hash".into(), json!("ABC123"));
        params.insert("binary".into(), json!(true));
        let req = GenericRequest::new("ledger", None, params);
        let value: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            value,
            json!({"command": "ledger", "ledger_hash": "ABC123", "binary": true})
        );
    }

    #[test]
    fn test_roundtrip_with_params_and_id() {
        let mut params = Map::new();
        params.insert(
            "account".into(),
            json!("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"),
        );
        let req = GenericRequest::new(
            Cow::Borrowed("account_info"),
            Some(Cow::Borrowed("id-42")),
            params.clone(),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: GenericRequest<'static> = serde_json::from_str(&serialized).unwrap();
        // The deserialized value is owned; compare by re-serializing.
        let re_serialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, re_serialized);
    }

    #[test]
    fn test_deserialize_missing_command_errors() {
        let json = r#"{"id": "x"}"#;
        let err = serde_json::from_str::<GenericRequest<'static>>(json).unwrap_err();
        assert!(err.to_string().contains("command"));
    }

    /// A caller who somehow gets a reserved key into `params` (e.g. via
    /// struct-literal construction that bypasses `new()`'s debug_assert,
    /// or by deserializing user-controlled JSON) must not be able to
    /// override the request's top-level `command` on the wire. In release
    /// builds `serialize` filters the reserved keys out of `params`.
    #[test]
    fn test_serialize_drops_reserved_command_key_in_params() {
        let mut params = Map::new();
        params.insert("command".into(), json!("stop"));
        params.insert("account".into(), json!("rAlice"));
        // Bypass `new()` (which debug-asserts) via struct-literal.
        let req = GenericRequest {
            common_fields: CommonFields {
                command: RequestMethod::Generic,
                id: None,
            },
            command: Cow::Borrowed("ledger_accept"),
            params,
        };
        let value: Value = serde_json::to_value(&req).unwrap();
        // The intended command wins and the injected "command" is gone.
        assert_eq!(value["command"], json!("ledger_accept"));
        assert_eq!(value["account"], json!("rAlice"));
        // No sign of the injected reserved key elsewhere.
        assert_eq!(value.as_object().unwrap().len(), 2);
    }

    #[test]
    fn test_serialize_drops_reserved_id_key_in_params() {
        let mut params = Map::new();
        params.insert("id".into(), json!("attacker-id"));
        let req = GenericRequest {
            common_fields: CommonFields {
                command: RequestMethod::Generic,
                id: Some(Cow::Borrowed("real-id")),
            },
            command: Cow::Borrowed("server_info"),
            params,
        };
        let value: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(value["id"], json!("real-id"));
        assert_eq!(value.as_object().unwrap().len(), 2);
    }

    // `debug_assert!` compiles out under `--release`, so these should_panic
    // tests are gated on `debug_assertions`. The serialize filter above is
    // what protects release builds (covered by the two tests just above).
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "reserved keys")]
    fn test_new_debug_asserts_on_reserved_command_key() {
        let mut params = Map::new();
        params.insert("command".into(), json!("stop"));
        let _ = GenericRequest::new("ledger_accept", None, params);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "reserved keys")]
    fn test_new_debug_asserts_on_reserved_id_key() {
        let mut params = Map::new();
        params.insert("id".into(), json!("x"));
        let _ = GenericRequest::new("ledger_accept", None, params);
    }

    #[test]
    fn test_common_fields_id_is_mutable_for_client_autofill() {
        let mut req = GenericRequest::new("ledger_accept", None, Map::new());
        assert!(req.get_common_fields().id.is_none());
        req.get_common_fields_mut().id = Some(Cow::Borrowed("auto-42"));
        let value: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(value, json!({"command": "ledger_accept", "id": "auto-42"}));
    }
}
