use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{marker::Marker, nft_offer::NFTOffer};
use crate::_serde::marker;

/// Response from an nft_buy_offers request, containing a list of buy offers
/// for a specific NFToken.
///
/// See Nft Buy Offers:
/// `<https://xrpl.org/nft_buy_offers.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NFTBuyOffers<'a> {
    /// The NFToken these offers are for, as specified in the request.
    pub nft_id: Cow<'a, str>,
    /// A list of buy offers for the token.
    pub offers: Cow<'a, [NFTOffer<'a>]>,
    /// The limit, as specified in the request.
    pub limit: Option<u32>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off. Omitted when
    /// there are no pages of information after this one.
    #[serde(with = "marker", default)]
    pub marker: Option<Marker>,
}

#[cfg(test)]
mod tests {
    use crate::models::Amount;

    use super::*;

    #[test]
    fn test_nft_buy_offers_deserialization() {
        let json = r#"{
            "nft_id": "00090000D0B007439B080E9B05BF62403911301A7B1F0CFAA048C0A200000007",
            "offers": [
                {
                    "amount": "1500",
                    "flags": 0,
                    "nft_offer_index": "3212D26DB00031889D4EF7D9129BB0FA673B5B40B1759564486C0F0946BA203F",
                    "owner": "rsuHaTvJh1bDmDoxX9QcKP7HEBSBt4XsHx"
                }
            ]
        }"#;

        let nft_buy_offers: NFTBuyOffers = serde_json::from_str(json).unwrap();

        assert_eq!(
            nft_buy_offers.nft_id,
            "00090000D0B007439B080E9B05BF62403911301A7B1F0CFAA048C0A200000007"
        );
        assert_eq!(nft_buy_offers.offers.len(), 1);

        let first_offer = &nft_buy_offers.offers[0];
        assert_eq!(first_offer.amount, Amount::XRPAmount("1500".into()));
        assert_eq!(
            u32::try_from(first_offer.flags.clone()).expect("Failed to convert flags to u32"),
            0
        );
        assert_eq!(
            first_offer.nft_offer_index,
            "3212D26DB00031889D4EF7D9129BB0FA673B5B40B1759564486C0F0946BA203F"
        );
        assert_eq!(first_offer.owner, "rsuHaTvJh1bDmDoxX9QcKP7HEBSBt4XsHx");
        assert_eq!(first_offer.expiration, None);
        assert_eq!(first_offer.destination, None);

        // Test serialization
        let serialized = serde_json::to_string(&nft_buy_offers).unwrap();
        let deserialized: NFTBuyOffers = serde_json::from_str(&serialized).unwrap();
        assert_eq!(nft_buy_offers, deserialized);
    }
}
