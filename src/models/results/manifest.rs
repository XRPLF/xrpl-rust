use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response from a manifest request, containing validator manifest information.
///
/// See Manifest:
/// `<https://xrpl.org/manifest.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Manifest<'a> {
    /// The data contained in this manifest. Omitted if the server does not
    /// have a manifest for the public_key from the request.
    pub details: Option<ManifestDetails<'a>>,
    /// The full manifest data in base64 format. This data is serialized to
    /// binary before being base64-encoded. Omitted if the server does not
    /// have a manifest for the public_key from the request.
    pub manifest: Option<Cow<'a, str>>,
    /// The public_key from the request.
    pub requested: Cow<'a, str>,
}

/// Details object containing the parsed contents of a validator manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ManifestDetails<'a> {
    /// The domain name this validator claims to be associated with.
    /// If the manifest does not contain a domain, this is an empty string.
    pub domain: Cow<'a, str>,
    /// The ephemeral public key for this validator, in base58.
    pub ephemeral_key: Cow<'a, str>,
    /// The master public key for this validator, in base58.
    pub master_key: Cow<'a, str>,
    /// The sequence number of this manifest. This number increases whenever
    /// the validator operator updates the validator's token to rotate
    /// ephemeral keys or change settings.
    pub seq: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialize() {
        let json = r#"{
            "details": {
                "domain": "",
                "ephemeral_key": "n9J67zk4B7GpbQV5jRQntbgdKf7TW6894QuG7qq1rE5gvjCu6snA",
                "master_key": "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p",
                "seq": 1
            },
            "manifest": "JAAAAAFxIe3AkJgOyqs3y+UuiAI27Ff3Mrfbt8e7mjdo06bnGEp5XnMhAhRmvCZmWZXlwShVE9qXs2AVCvhVuA/WGYkTX/vVGBGwdkYwRAIgGnYpIGufURojN2cTXakAM7Vwa0GR7o3osdVlZShroXQCIH9R/Lx1v9rdb4YY2n5nrxdnhSSof3U6V/wIHJmeao5ucBJA9D1iAMo7YFCpb245N3Czc0L1R2Xac0YwQ6XdGT+cZ7yw2n8JbdC3hH8Xu9OUqc867Ee6JmlXtyDHzBdY/hdJCQ==",
            "requested": "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p"
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();

        // Test top-level fields
        assert_eq!(
            manifest.requested,
            "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p"
        );
        assert_eq!(
            manifest.manifest.unwrap(),
            "JAAAAAFxIe3AkJgOyqs3y+UuiAI27Ff3Mrfbt8e7mjdo06bnGEp5XnMhAhRmv\
            CZmWZXlwShVE9qXs2AVCvhVuA/WGYkTX/vVGBGwdkYwRAIgGnYpIGufURojN2c\
            TXakAM7Vwa0GR7o3osdVlZShroXQCIH9R/Lx1v9rdb4YY2n5nrxdnhSSof3U6V\
            /wIHJmeao5ucBJA9D1iAMo7YFCpb245N3Czc0L1R2Xac0YwQ6XdGT+cZ7yw2n8\
            JbdC3hH8Xu9OUqc867Ee6JmlXtyDHzBdY/hdJCQ=="
        );

        // Test details object
        let details = manifest.details.unwrap();
        assert_eq!(details.domain, "");
        assert_eq!(
            details.ephemeral_key,
            "n9J67zk4B7GpbQV5jRQntbgdKf7TW6894QuG7qq1rE5gvjCu6snA"
        );
        assert_eq!(
            details.master_key,
            "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p"
        );
        assert_eq!(details.seq, 1);
    }

    #[test]
    fn test_manifest_serialize() {
        let manifest = Manifest {
            details: Some(ManifestDetails {
                domain: "".into(),
                ephemeral_key: "n9J67zk4B7GpbQV5jRQntbgdKf7TW6894QuG7qq1rE5gvjCu6snA".into(),
                master_key: "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p".into(),
                seq: 1,
            }),
            manifest: Some(
                "JAAAAAFxIe3AkJgOyqs3y+UuiAI27Ff3Mrfbt8e7mjdo06b\
                nGEp5XnMhAhRmvCZmWZXlwShVE9qXs2AVCvhVuA/WGYkTX/vVGBGwdkYwRAIgGn\
                YpIGufURojN2cTXakAM7Vwa0GR7o3osdVlZShroXQCIH9R/Lx1v9rdb4YY2n5nr\
                xdnhSSof3U6V/wIHJmeao5ucBJA9D1iAMo7YFCpb245N3Czc0L1R2Xac0YwQ6Xd\
                GT+cZ7yw2n8JbdC3hH8Xu9OUqc867Ee6JmlXtyDHzBdY/hdJCQ=="
                    .into(),
            ),
            requested: "nHUFE9prPXPrHcG3SkwP1UzAQbSphqyQkQK9ATXLZsfkezhhda3p".into(),
        };

        let serialized = serde_json::to_string(&manifest).unwrap();
        let deserialized: Manifest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
