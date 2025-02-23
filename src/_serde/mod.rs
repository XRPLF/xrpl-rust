//! Serde functionalities

use crate::models::FlagCollection;
use alloc::format;
use alloc::vec::Vec;
use core::hash::BuildHasherDefault;
use core::{convert::TryFrom, fmt::Debug};
use fnv::FnvHasher;
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use strum::IntoEnumIterator;

pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FnvHasher>>;

fn serialize_flag<F, S>(flags: &FlagCollection<F>, s: S) -> Result<S::Ok, S::Error>
where
    F: Serialize + IntoEnumIterator,
    S: Serializer,
{
    let flags_value_result: Result<Value, serde_json::Error> = serde_json::to_value(flags);
    match flags_value_result {
        Ok(flags_as_value) => {
            let flag_vec_result: Result<Vec<u32>, serde_json::Error> =
                serde_json::from_value(flags_as_value);
            match flag_vec_result {
                Ok(flags_vec) => s.serialize_u32(flags_vec.iter().sum()),
                Err(_) => {
                    // TODO: Find a way to use custom errors
                    Err(ser::Error::custom("SerdeIntermediateStepError: Failed to turn flags into `Vec<u32>` during serialization"))
                }
            }
        }
        Err(_) => Err(ser::Error::custom(
            "SerdeIntermediateStepError: Failed to turn flags into `Value` during serialization",
        )),
    }
}

fn deserialize_flags<'de, D, F>(d: D) -> Result<FlagCollection<F>, D::Error>
where
    F: Serialize + IntoEnumIterator + Debug,
    D: Deserializer<'de>,
{
    let flags_u32 = u32::deserialize(d)?;
    FlagCollection::<F>::try_from(flags_u32).map_err(|_e| {
        de::Error::custom(format!(
            "SerdeIntermediateStepError: Failed to turn `u32` into `FlagCollection<{}>` during deserialization",
            core::any::type_name::<F>()
        ))
    })
}

/// A `mod` to be used on transaction `flags` fields. It serializes the `Vec<Flag>` into a `u32`,
/// representing the bit-flags, and deserializes the `u32` back into `Vec<Flag>` for internal uses.
pub(crate) mod txn_flags {
    use core::fmt::Debug;

    use crate::_serde::{deserialize_flags, serialize_flag};

    use serde::{Deserializer, Serialize, Serializer};

    use crate::models::FlagCollection;
    use strum::IntoEnumIterator;

    pub fn serialize<F, S>(flags: &FlagCollection<F>, s: S) -> Result<S::Ok, S::Error>
    where
        F: Serialize + IntoEnumIterator + Debug,
        S: Serializer,
    {
        if flags.0.is_empty() {
            s.serialize_u32(0)
        } else {
            serialize_flag(flags, s)
        }
    }

    pub fn deserialize<'de, F, D>(d: D) -> Result<FlagCollection<F>, D::Error>
    where
        F: Serialize + IntoEnumIterator + Debug,
        D: Deserializer<'de>,
    {
        let flags_vec_result: Result<FlagCollection<F>, D::Error> = deserialize_flags(d);
        match flags_vec_result {
            Ok(flags_vec) => {
                if flags_vec.0.is_empty() {
                    Ok(FlagCollection::<F>::default())
                } else {
                    Ok(flags_vec)
                }
            }
            Err(error) => Err(error),
        }
    }
}

pub(crate) mod lgr_obj_flags {
    use core::fmt::Debug;

    use crate::_serde::{deserialize_flags, serialize_flag};
    use crate::models::FlagCollection;
    use serde::{Deserializer, Serialize, Serializer};
    use strum::IntoEnumIterator;

    pub fn serialize<F, S>(flags: &FlagCollection<F>, s: S) -> Result<S::Ok, S::Error>
    where
        F: Serialize + IntoEnumIterator,
        S: Serializer,
    {
        if !flags.0.is_empty() {
            serialize_flag(flags, s)
        } else {
            s.serialize_u32(0)
        }
    }

    pub fn deserialize<'de, F, D>(d: D) -> Result<FlagCollection<F>, D::Error>
    where
        F: Serialize + IntoEnumIterator + Debug,
        D: Deserializer<'de>,
    {
        deserialize_flags(d)
    }
}

/// A module for serializing and deserializing `Marker` objects.
/// This allows a `Marker` to be deserialized from either a string or an object.
pub(crate) mod marker {
    use alloc::string::String;
    use core::fmt;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    use crate::models::results::marker::Marker;

    pub fn serialize<S>(marker: &Option<Marker>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match marker {
            Some(m) => m.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Marker>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // This visitor handles both string and struct deserialization
        struct MarkerVisitor;

        impl<'de> de::Visitor<'de> for MarkerVisitor {
            type Value = Option<Marker>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, a Marker object, or null")
            }

            // Handle null/none
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            // Handle string deserialization
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(Marker::from(value)))
            }

            // Handle string deserialization
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(Marker::from(value)))
            }

            // Handle object deserialization
            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                // Use the default struct deserializer
                let deserializer = de::value::MapAccessDeserializer::new(map);
                #[derive(Deserialize)]
                struct MarkerHelper {
                    ledger: u32,
                    seq: u32,
                }

                let helper = MarkerHelper::deserialize(deserializer)?;
                Ok(Some(Marker {
                    ledger: helper.ledger,
                    seq: helper.seq,
                }))
            }
        }

        // Try to deserialize as either a string, an object, or null
        deserializer.deserialize_any(MarkerVisitor)
    }
}

/// A macro to tag a struct externally. With `serde` attributes, unfortunately it is not possible to
/// serialize a struct to json with its name as `key` and its fields as `value`. Example:
/// `{"Example":{"Field1":"hello","Field2":"world"}}`
///
/// Several models need to be serialized in that format. This macro uses a helper to serialize and
/// deserialize to/from that format.
///
/// Resource: https://github.com/serde-rs/serde/issues/554#issuecomment-249211775
// TODO: Find a way to `#[skip_serializing_none]`
#[macro_export]
macro_rules! serde_with_tag {
    (
        $(#[$attr:meta])*
        pub struct $name:ident<$lt:lifetime> {
            $(
                $(#[$doc:meta])*
                pub $field:ident: $ty:ty,
            )*
        }
    ) => {
        $(#[$attr])*
        pub struct $name<$lt> {
            $(
                $(#[$doc])*
                pub $field: $ty,
            )*
        }

        impl<$lt> ::serde::Serialize for $name<$lt> {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer
            {
                #[derive(::serde::Serialize)]
                #[serde(rename_all = "PascalCase")]
                #[::serde_with::skip_serializing_none]
                struct Helper<$lt> {
                    $(
                        $field: $ty,
                    )*
                }

                let helper = Helper {
                    $(
                        $field: self.$field.clone(),
                    )*
                };

                let mut state = serializer.serialize_map(Some(1))?;
                state.serialize_key(stringify!($name))?;
                state.serialize_value(&helper)?;
                state.end()
            }
        }

        impl<'de: $lt, $lt> ::serde::Deserialize<'de> for $name<$lt> {
            #[allow(non_snake_case)]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(rename_all = "PascalCase")]
                #[::serde_with::skip_serializing_none]
                struct Helper<$lt> {
                    $(
                        $field: $ty,
                    )*
                }

                let hash_map: $crate::_serde::HashMap<&$lt str, Helper<$lt>> = $crate::_serde::HashMap::deserialize(deserializer)?;
                let helper_result = hash_map.get(stringify!($name));

                match helper_result {
                    Some(helper) => {
                        Ok(Self {
                            $(
                                $field: helper.$field.clone().into(),
                            )*
                        })
                    }
                    None => {
                        Err(::serde::de::Error::custom("SerdeIntermediateStepError: Unable to find model name as json key."))
                    }
                }
            }
        }
    };
    (
        $(#[$attr:meta])*
        pub struct $name:ident {
            $(
                $(#[$doc:meta])*
                pub $field:ident: $ty:ty,
            )*
        }
    ) => {
        $(#[$attr])*
        pub struct $name {
            $(
                $(#[$doc])*
                pub $field: $ty,
            )*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer
            {
                #[derive(::serde::Serialize)]
                #[serde(rename_all = "PascalCase")]
                #[::serde_with::skip_serializing_none]
                struct Helper {
                    $(
                        $field: $ty,
                    )*
                }

                let helper = Helper {
                    $(
                        $field: self.$field.clone(),
                    )*
                };

                let mut state = serializer.serialize_map(Some(1))?;
                state.serialize_key(stringify!($name))?;
                state.serialize_value(&helper)?;
                state.end()
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            #[allow(non_snake_case)]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(rename_all = "PascalCase")]
                #[::serde_with::skip_serializing_none]
                struct Helper {
                    $(
                        $field: $ty,
                    )*
                }

                let hash_map: $crate::_serde::HashMap<&'de str, Helper> = $crate::_serde::HashMap::deserialize(deserializer)?;
                let helper_result = hash_map.get(stringify!($name));

                match helper_result {
                    Some(helper) => {
                        Ok(Self {
                            $(
                                $field: helper.$field.clone().into(),
                            )*
                        })
                    }
                    None => {
                        Err(::serde::de::Error::custom("SerdeIntermediateStepError: Unable to find model name as json key."))
                    }
                }
            }
        }
    };
}
