use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::_serde::lgr_obj_flags;
use crate::models::ledger::objects::nftoken_offer::NFTokenOfferFlag;
use crate::models::{Amount, FlagCollection};

/// Represents a single buy offer for an NFToken.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NFTOffer<'a> {
    /// The amount offered to buy the NFToken.
    pub amount: Amount<'a>,
    /// Bit-map of boolean flags enabled for this offer.
    #[serde(with = "lgr_obj_flags")]
    pub flags: FlagCollection<NFTokenOfferFlag>,
    /// The unique identifier of this offer in the ledger.
    pub nft_offer_index: Cow<'a, str>,
    /// The account that placed this offer.
    pub owner: Cow<'a, str>,
    /// The time after which this offer is no longer valid, in seconds since
    /// the Ripple Epoch.
    pub expiration: Option<u64>,
    /// If present, this offer can only be accepted by this account.
    pub destination: Option<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CASES: &[(&str, u32)] = &[
        (
            r#"{
                "amount": "1000",
                "flags": 0,
                "nft_offer_index": "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1",
                "owner": "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt"
            }"#,
            0,
        ),
        (
            r#"{
                "amount": "1000",
                "flags": 1,
                "nft_offer_index": "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1",
                "owner": "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt"
            }"#,
            1,
        ),
    ];

    #[test]
    fn test_nft_offer_deserialization() {
        for (json, expected_flags) in TEST_CASES {
            let offer: NFTOffer = serde_json::from_str(json).unwrap();

            assert_eq!(offer.amount, Amount::XRPAmount("1000".into()));
            assert_eq!(u32::try_from(offer.flags.clone()).unwrap(), *expected_flags);
            assert_eq!(
                offer.nft_offer_index,
                "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1"
            );
            assert_eq!(offer.owner, "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt");
            assert_eq!(offer.expiration, None);
            assert_eq!(offer.destination, None);

            // Test serialization
            let serialized = serde_json::to_string(&offer).unwrap();
            let deserialized: NFTOffer = serde_json::from_str(&serialized).unwrap();
            assert_eq!(offer, deserialized);
        }
    }

    #[test]
    fn test_nft_offer_with_optional_fields() {
        let json = r#"{
            "amount": "1000",
            "flags": 1,
            "nft_offer_index": "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1",
            "owner": "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt",
            "expiration": 123456789,
            "destination": "rDestinationAccountAddress"
        }"#;

        let offer: NFTOffer = serde_json::from_str(json).unwrap();

        assert_eq!(offer.amount, Amount::XRPAmount("1000".into()));
        assert_eq!(u32::try_from(offer.flags.clone()).unwrap(), 1);
        assert_eq!(
            offer.nft_offer_index,
            "9E28E366573187F8E5B85CE301F229E061A619EE5A589EF740088F8843BF10A1"
        );
        assert_eq!(offer.owner, "rLpSRZ1E8JHyNDZeHYsQs1R5cwDCB3uuZt");
        assert_eq!(offer.expiration, Some(123456789));
        assert_eq!(offer.destination, Some("rDestinationAccountAddress".into()));

        // Test serialization
        let serialized = serde_json::to_string(&offer).unwrap();
        let deserialized: NFTOffer = serde_json::from_str(&serialized).unwrap();
        assert_eq!(offer, deserialized);
    }
}
