use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::requests::Marker;

use super::nft_offer::NFTOffer;

/// Response from an nft_sell_offers request, containing a list of sell offers
/// for a specific NFToken.
///
/// See Nft Sell Offers:
/// `<https://xrpl.org/nft_sell_offers.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NFTSellOffers<'a> {
    /// The NFToken these offers are for, as specified in the request.
    pub nft_id: Cow<'a, str>,
    /// A list of sell offers for the token.
    pub offers: Cow<'a, [NFTOffer<'a>]>,
    /// The limit, as specified in the request.
    pub limit: Option<u32>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off. Omitted when
    /// there are no pages of information after this one.
    pub marker: Option<Marker<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Amount;

    #[test]
    fn test_nft_sell_offers_deserialization() {
        let json = r#"{
            "nft_id": "00090000D0B007439B080E9B05BF62403911301A7B1F0CFAA048C0A200000007",
            "offers": [
                {
                    "amount": "1000",
                    "flags": 1,
                    "nft_offer_index": "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1",
                    "owner": "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt"
                }
            ]
        }"#;

        let nft_sell_offers: NFTSellOffers = serde_json::from_str(json).unwrap();

        assert_eq!(
            nft_sell_offers.nft_id,
            "00090000D0B007439B080E9B05BF62403911301A7B1F0CFAA048C0A200000007"
        );
        assert_eq!(nft_sell_offers.offers.len(), 1);

        let first_offer = &nft_sell_offers.offers[0];
        assert_eq!(first_offer.amount, Amount::XRPAmount("1000".into()));
        assert_eq!(u32::try_from(first_offer.flags.clone()).unwrap(), 1);
        assert_eq!(
            first_offer.nft_offer_index,
            "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1"
        );
        assert_eq!(first_offer.owner, "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt");
        assert_eq!(first_offer.expiration, None);
        assert_eq!(first_offer.destination, None);

        // Test serialization
        let serialized = serde_json::to_string(&nft_sell_offers).unwrap();
        let deserialized: NFTSellOffers = serde_json::from_str(&serialized).unwrap();
        assert_eq!(nft_sell_offers, deserialized);
    }
}
