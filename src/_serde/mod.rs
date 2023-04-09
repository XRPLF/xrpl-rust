//! Serde functionalities

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::hash::BuildHasherDefault;
use fnv::FnvHasher;
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use strum::IntoEnumIterator;

pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FnvHasher>>;

fn serialize_flag<F, S>(flags: &Vec<F>, s: S) -> Result<S::Ok, S::Error>
where
    F: Serialize,
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

fn deserialize_flags<'de, D, F>(d: D) -> Result<Vec<F>, D::Error>
where
    F: Serialize + IntoEnumIterator + Debug,
    D: Deserializer<'de>,
{
    let flags_u32 = u32::deserialize(d)?;

    let mut flags_vec = Vec::new();
    for flag in F::iter() {
        let check_flag_string_result: Result<String, serde_json::Error> =
            serde_json::to_string(&flag);
        match check_flag_string_result {
            Ok(check_flag_string) => {
                let check_flag_u32_result = check_flag_string.parse::<u32>();
                match check_flag_u32_result {
                    Ok(check_flag) => {
                        if check_flag & flags_u32 == check_flag {
                            flags_vec.push(flag);
                        } else {
                            continue;
                        }
                    }
                    Err(_) => {
                        return Err(de::Error::custom("SerdeIntermediateStepError: Failed to turn flag into `u32` during deserialization"));
                    }
                };
            }
            Err(_) => {
                return Err(de::Error::custom("SerdeIntermediateStepError: Failed to turn flag into `String` during deserialization"));
            }
        };
    }

    Ok(flags_vec)
}

/// A `mod` to be used on transaction `flags` fields. It serializes the `Vec<Flag>` into a `u32`,
/// representing the bit-flags, and deserializes the `u32` back into `Vec<Flag>` for internal uses.
pub(crate) mod txn_flags {
    use core::fmt::Debug;

    use crate::_serde::{deserialize_flags, serialize_flag};
    use alloc::vec::Vec;

    use serde::{Deserializer, Serialize, Serializer};

    use strum::IntoEnumIterator;

    pub fn serialize<F, S>(flags: &Option<Vec<F>>, s: S) -> Result<S::Ok, S::Error>
    where
        F: Serialize,
        S: Serializer,
    {
        if let Some(f) = flags {
            serialize_flag(f, s)
        } else {
            s.serialize_u32(0)
        }
    }

    pub fn deserialize<'de, F, D>(d: D) -> Result<Option<Vec<F>>, D::Error>
    where
        F: Serialize + IntoEnumIterator + Debug,
        D: Deserializer<'de>,
    {
        let flags_vec_result: Result<Vec<F>, D::Error> = deserialize_flags(d);
        match flags_vec_result {
            Ok(flags_vec) => {
                if flags_vec.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(flags_vec))
                }
            }
            Err(error) => Err(error),
        }
    }
}

pub(crate) mod lgr_obj_flags {
    use core::fmt::Debug;

    use crate::_serde::{deserialize_flags, serialize_flag};
    use alloc::vec::Vec;
    use serde::{Deserializer, Serialize, Serializer};
    use strum::IntoEnumIterator;

    pub fn serialize<F, S>(flags: &Vec<F>, s: S) -> Result<S::Ok, S::Error>
    where
        F: Serialize,
        S: Serializer,
    {
        if !flags.is_empty() {
            serialize_flag(flags, s)
        } else {
            s.serialize_u32(0)
        }
    }

    pub fn deserialize<'de, F, D>(d: D) -> Result<Vec<F>, D::Error>
    where
        F: Serialize + IntoEnumIterator + Debug,
        D: Deserializer<'de>,
    {
        deserialize_flags(d)
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
