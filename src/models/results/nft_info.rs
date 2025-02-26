use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// See NFT Info:
/// `<https://xrpl.org/nft-info.html>`
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTInfo<'a> {
    #[serde(flatten)]
    pub base: NFToken<'a>,
    /// Whether this data is from a validated ledger version
    pub validated: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFToken<'a> {
    /// The unique identifier of the NFToken
    pub nft_id: Cow<'a, str>,
    /// The ledger index of the ledger that was current when this data was
    /// retrieved
    pub ledger_index: u32,
    /// The account that currently owns this NFToken
    pub owner: Cow<'a, str>,
    /// Whether this NFToken has been burned
    pub is_burned: bool,
    /// Bit-map of boolean flags enabled for this NFToken
    pub flags: u32,
    /// The transfer fee associated with this NFToken, in units of
    /// 1/10000 of 1%
    pub transfer_fee: u32,
    /// The account that issued this NFToken
    pub issuer: Cow<'a, str>,
    /// The taxon associated with this NFToken
    pub nft_taxon: u32,
    /// The serial number of this NFToken within its taxon
    pub nft_serial: u32,
    /// The URI data associated with this NFToken
    pub uri: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_info_deserialization() {
        let json = r#"{
            "nft_id": "00080000B4F4AFC5FBCBD76873F18006173D2193467D3EE70000099B00000000",
            "ledger_index": 270,
            "owner": "rG9gdNygQ6npA9JvDFWBoeXbiUcTYJnEnk",
            "is_burned": true,
            "flags": 8,
            "transfer_fee": 0,
            "issuer": "rHVokeuSnjPjz718qdb47bGXBBHNMP3KDQ",
            "nft_taxon": 0,
            "nft_serial": 0,
            "uri": "",
            "validated": true
        }"#;

        let nft_info: NFTInfo = serde_json::from_str(json).unwrap();

        assert_eq!(
            nft_info.base.nft_id,
            "00080000B4F4AFC5FBCBD76873F18006173D2193467D3EE70000099B00000000"
        );
        assert_eq!(nft_info.base.ledger_index, 270);
        assert_eq!(nft_info.base.owner, "rG9gdNygQ6npA9JvDFWBoeXbiUcTYJnEnk");
        assert!(nft_info.base.is_burned);
        assert_eq!(nft_info.base.flags, 8);
        assert_eq!(nft_info.base.transfer_fee, 0);
        assert_eq!(nft_info.base.issuer, "rHVokeuSnjPjz718qdb47bGXBBHNMP3KDQ");
        assert_eq!(nft_info.base.nft_taxon, 0);
        assert_eq!(nft_info.base.nft_serial, 0);
        assert_eq!(nft_info.base.uri, "");
        assert_eq!(nft_info.validated, Some(true));

        // Test serialization
        let serialized = serde_json::to_string(&nft_info).unwrap();
        let deserialized: NFTInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(nft_info, deserialized);
    }
}
