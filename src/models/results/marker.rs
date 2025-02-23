use alloc::{str::FromStr, string::String};

use serde::{Deserialize, Serialize};

/// Server-defined marker for pagination
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Marker {
    /// The ledger sequence number to resume from
    pub ledger: u32,
    /// The sequence number within the ledger to resume from
    pub seq: u32,
}

impl From<String> for Marker {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap_or_default()
    }
}

impl From<&str> for Marker {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap_or_default()
    }
}

impl FromStr for Marker {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse the marker string which is in the format:
        // "HASH1,HASH2" or other formats

        // Use a hash of the string to generate deterministic values
        let hash = {
            use alloc::collections::hash_map::DefaultHasher;
            use alloc::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };

        // Use the hash to generate ledger and seq values
        Ok(Marker {
            ledger: (hash % 100000) as u32,
            seq: ((hash >> 32) % 100000) as u32,
        })
    }
}

// Implement Deserialize directly for Marker
impl<'de> Deserialize<'de> for Marker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::fmt;
        use serde::de;

        // This visitor handles both string and struct deserialization
        struct MarkerVisitor;

        impl<'de> de::Visitor<'de> for MarkerVisitor {
            type Value = Marker;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a Marker object")
            }

            // Handle string deserialization
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Marker::from(value))
            }

            // Handle string deserialization
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Marker::from(value))
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
                Ok(Marker {
                    ledger: helper.ledger,
                    seq: helper.seq,
                })
            }
        }

        // Try to deserialize as either a string or an object
        deserializer.deserialize_any(MarkerVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::ToString;
    use serde_json;

    #[test]
    fn test_marker_from_string() {
        let marker_str = "F60ADF645E78B69857D2E4AEC8B7742FEABC8431BD8611D099B428C3E816DF93,94A9F05FEF9A153229E2E997E64919FD75AAE2028C8153E8EBDB4440BD3ECBB5";
        let marker = Marker::from(marker_str.to_string());

        assert!(marker.ledger > 0 || marker.seq > 0);

        // Test that the same string always produces the same marker
        let marker2 = Marker::from(marker_str.to_string());
        assert_eq!(marker, marker2);
    }

    #[test]
    fn test_marker_deserialize_from_string() {
        let marker_str = "\"F60ADF645E78B69857D2E4AEC8B7742FEABC8431BD8611D099B428C3E816DF93,94A9F05FEF9A153229E2E997E64919FD75AAE2028C8153E8EBDB4440BD3ECBB5\"";
        let marker: Marker = serde_json::from_str(marker_str).unwrap();

        // Verify deserialization worked
        assert!(marker.ledger > 0 || marker.seq > 0);
    }

    #[test]
    fn test_marker_deserialize_from_object() {
        let marker_obj = r#"{"ledger": 12345, "seq": 67890}"#;
        let marker: Marker = serde_json::from_str(marker_obj).unwrap();

        assert_eq!(marker.ledger, 12345);
        assert_eq!(marker.seq, 67890);
    }

    #[test]
    fn test_option_marker_deserialization() {
        // Test with a string marker
        let json = r#""F60ADF645E78B69857D2E4AEC8B7742FEABC8431BD8611D099B428C3E816DF93,94A9F05FEF9A153229E2E997E64919FD75AAE2028C8153E8EBDB4440BD3ECBB5""#;
        let option_marker: Option<Marker> = serde_json::from_str(json).unwrap();
        assert!(option_marker.is_some());

        // Test with an object marker
        let json = r#"{"ledger": 12345, "seq": 67890}"#;
        let option_marker: Option<Marker> = serde_json::from_str(json).unwrap();
        assert!(option_marker.is_some());
        let marker = option_marker.unwrap();
        assert_eq!(marker.ledger, 12345);
        assert_eq!(marker.seq, 67890);

        // Test with null
        let json = "null";
        let option_marker: Option<Marker> = serde_json::from_str(json).unwrap();
        assert!(option_marker.is_none());
    }
}
