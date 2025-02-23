use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response from an account_nfts request, containing a list of NFTs owned by
/// the specified account.
///
/// See Account NFTs:
/// `<https://xrpl.org/account_nfts.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountNfts<'a> {
    /// The account that owns the list of NFTs.
    pub account: Cow<'a, str>,
    /// A list of NFTs owned by the account, formatted as NFT Objects.
    #[serde(rename = "account_nfts")]
    pub nfts: Cow<'a, [NFToken<'a>]>,
    /// (May be omitted) The identifying hash of the ledger that was used to
    /// generate this response.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// (May be omitted) The ledger index of the ledger that was used to
    /// generate this response.
    pub ledger_index: u32,
    /// (May be omitted) The ledger index of the current in-progress ledger
    /// version, which was used to generate this response.
    pub ledger_current_index: Option<u32>,
    /// If included and set to true, the information in this response comes
    /// from a validated ledger version. Otherwise, the information is
    /// subject to change.
    pub validated: bool,
    /// The limit, as specified in the request.
    pub limit: Option<u32>,
    /// (May be omitted) Server-defined value indicating the response is
    /// paginated. Pass this to the next call to resume where this call
    /// left off. Omitted when there are no additional pages after this one.
    pub marker: Option<Value>,
}

/// Each object in the account_nfts array represents one NFToken.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NFToken<'a> {
    /// A bit-map of boolean flags enabled for this NFToken.
    /// See NFToken Flags for possible values.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// The account that issued this NFToken.
    #[serde(rename = "Issuer")]
    pub issuer: Cow<'a, str>,
    /// The unique identifier of this NFToken, in hexadecimal.
    #[serde(rename = "NFTokenID")]
    pub nft_id: Cow<'a, str>,
    /// The token sequence number of this NFToken, which is unique for
    /// its issuer.
    pub nft_serial: u32,
    /// The unscrambled version of this token's taxon. Several tokens with
    /// the same taxon might represent instances of a limited series.
    #[serde(rename = "NFTokenTaxon")]
    pub token_taxon: u32,
    /// The URI data associated with this NFToken, in hexadecimal.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_nfts_deserialization() {
        let json = r#"{
            "account": "rsuHaTvJh1bDmDoxX9QcKP7HEBSBt4XsHx",
            "account_nfts": [
                {
                    "Flags": 1,
                    "Issuer": "rGJUF4PvVkMNxG6Bg6AKg3avhrtQyAffcm",
                    "NFTokenID": "00010000A7CAD27B688D14BA1A9FA5366554D6ADCF9CE0875B974D9F00000004",
                    "NFTokenTaxon": 0,
                    "URI": "697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469",
                    "nft_serial": 4
                },
                {
                    "Flags": 1,
                    "Issuer": "rGJUF4PvVkMNxG6Bg6AKg3avhrtQyAffcm",
                    "NFTokenID": "00010000A7CAD27B688D14BA1A9FA5366554D6ADCF9CE087727D1EA000000005",
                    "NFTokenTaxon": 0,
                    "URI": "697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469",
                    "nft_serial": 5
                }
            ],
            "ledger_hash": "46497E9FF17A993324F1A0A693DC068B467184023C7FD162812265EAAFEB97CB",
            "ledger_index": 2380559,
            "status": "success",
            "validated": true
        }"#;

        let account_nfts: AccountNfts = serde_json::from_str(json).unwrap();

        // Test main struct fields
        assert_eq!(account_nfts.account, "rsuHaTvJh1bDmDoxX9QcKP7HEBSBt4XsHx");
        assert_eq!(
            account_nfts.ledger_hash.unwrap(),
            "46497E9FF17A993324F1A0A693DC068B467184023C7FD162812265EAAFEB97CB"
        );
        assert_eq!(account_nfts.ledger_index, 2380559);
        assert!(account_nfts.validated);
        assert_eq!(account_nfts.nfts.len(), 2);

        // Test first NFT
        let first_nft = &account_nfts.nfts[0];
        assert_eq!(first_nft.flags, 1);
        assert_eq!(first_nft.issuer, "rGJUF4PvVkMNxG6Bg6AKg3avhrtQyAffcm");
        assert_eq!(
            first_nft.nft_id,
            "00010000A7CAD27B688D14BA1A9FA5366554D6ADCF9CE0875B974D9F00000004"
        );
        assert_eq!(first_nft.token_taxon, 0);
        assert_eq!(first_nft.nft_serial, 4);
        assert_eq!(
            first_nft.uri.as_ref().unwrap(),
            "697066733A2F2F62616679626569676479727A7435736670\
            3775646D37687537367568377932366E6634646675796C716\
            16266336F636C67747179353566627A6469"
        );

        // Test second NFT
        let second_nft = &account_nfts.nfts[1];
        assert_eq!(second_nft.flags, 1);
        assert_eq!(second_nft.issuer, "rGJUF4PvVkMNxG6Bg6AKg3avhrtQyAffcm");
        assert_eq!(
            second_nft.nft_id,
            "00010000A7CAD27B688D14BA1A9FA5366554D6ADCF9CE087727D1EA000000005"
        );
        assert_eq!(second_nft.token_taxon, 0);
        assert_eq!(second_nft.nft_serial, 5);
        assert_eq!(
            second_nft.uri.as_ref().unwrap(),
            "697066733A2F2F62616679626569676479727A74357366703\
            775646D37687537367568377932366E6634646675796C71616\
            266336F636C67747179353566627A6469"
        );
    }
}
