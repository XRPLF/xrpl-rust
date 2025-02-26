use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::Amount;

/// Response format for the amm_info method, which returns information about
/// an Automated Market Maker (AMM) instance.
///
/// See AMM Info:
/// `<https://xrpl.org/amm_info.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AMMInfo<'a> {
    /// The AMM Description Object for the requested asset pair.
    pub amm: AMMDescription<'a>,
    /// The ledger index of the current in-progress ledger.
    /// Omitted if ledger_index is provided instead.
    pub ledger_current_index: Option<u32>,
    /// The identifying hash of the ledger version used.
    /// Omitted if ledger_current_index is provided instead.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version used.
    /// Omitted if ledger_current_index is provided instead.
    pub ledger_index: Option<u32>,
    /// If true, the ledger is validated and results are final.
    pub validated: Option<bool>,
}

/// Describes the current status of an Automated Market Maker (AMM) in
/// the ledger.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AMMDescription<'a> {
    /// The Address of the AMM Account.
    pub account: Cow<'a, str>,
    /// The total amount of one asset in the AMM's pool.
    pub amount: Amount<'a>,
    /// The total amount of the other asset in the AMM's pool.
    pub amount2: Amount<'a>,
    /// If true, the amount currency is currently frozen. Omitted for XRP.
    pub asset_frozen: Option<bool>,
    /// If true, the amount2 currency is currently frozen. Omitted for XRP.
    pub asset2_frozen: Option<bool>,
    /// Details about the current auction slot holder, if there is one.
    pub auction_slot: Option<AuctionSlot<'a>>,
    /// The total amount of this AMM's LP Tokens outstanding.
    /// If a liquidity provider was specified, this is their LP Token balance.
    pub lp_token: Amount<'a>,
    /// The AMM's current trading fee, in units of 1/100,000.
    pub trading_fee: u32,
    /// The current votes for the AMM's trading fee.
    pub vote_slots: Option<Cow<'a, [VoteSlot<'a>]>>,
}

/// Describes the current auction slot holder of the AMM.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AuctionSlot<'a> {
    /// The Address of the account that owns the auction slot.
    pub account: Cow<'a, str>,
    /// Additional accounts eligible for the discounted trading fee.
    pub auth_accounts: Option<Cow<'a, [AuthAccount<'a>]>>,
    /// The discounted trading fee (1/10 of the AMM's normal trading fee).
    pub discounted_fee: u32,
    /// The ISO 8601 UTC timestamp when this auction slot expires.
    pub expiration: Cow<'a, str>,
    /// The amount in LP Tokens paid to win the auction slot.
    pub price: Amount<'a>,
    /// Current 72-minute time interval (0-19).
    pub time_interval: u32,
}

/// Represents an authorized account for discounted trading.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AuthAccount<'a> {
    /// The address of the designated account.
    pub account: Cow<'a, str>,
}

/// Represents one liquidity provider's vote to set the trading fee.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VoteSlot<'a> {
    /// The Address of this liquidity provider.
    pub account: Cow<'a, str>,
    /// The trading fee voted for, in units of 1/100,000.
    pub trading_fee: u32,
    /// The vote weight, proportional to LP Token holdings.
    pub vote_weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_info_deserialize() {
        let json = r#"{
            "amm": {
                "account": "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM",
                "amount": "296890496",
                "amount2": {
                    "currency": "TST",
                    "issuer": "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd",
                    "value": "25.81656470648473"
                },
                "asset2_frozen": false,
                "auction_slot": {
                    "account": "rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm",
                    "auth_accounts": [
                        {
                            "account": "r3f2WpQMsAd8k4Zoijv2PZ78EYFJ2EdvgV"
                        },
                        {
                            "account": "rnW8FAPgpQgA6VoESnVrUVJHBdq9QAtRZs"
                        }
                    ],
                    "discounted_fee": 60,
                    "expiration": "2023-Jan-26 00:28:40.000000000 UTC",
                    "price": {
                        "currency": "039C99CD9AB0B70B32ECDA51EAAE471625608EA2",
                        "issuer": "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM",
                        "value": "0"
                    },
                    "time_interval": 0
                },
                "lp_token": {
                    "currency": "039C99CD9AB0B70B32ECDA51EAAE471625608EA2",
                    "issuer": "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM",
                    "value": "87533.41976112682"
                },
                "trading_fee": 600,
                "vote_slots": [
                    {
                        "account": "rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm",
                        "trading_fee": 600,
                        "vote_weight": 9684
                    }
                ]
            },
            "ledger_current_index": 316725,
            "validated": false
        }"#;

        let amm_info: AMMInfo = serde_json::from_str(json).unwrap();

        // Verify the top-level fields
        assert_eq!(amm_info.ledger_current_index, Some(316725));
        assert_eq!(amm_info.validated, Some(false));
        assert!(amm_info.ledger_hash.is_none());
        assert!(amm_info.ledger_index.is_none());

        // Verify AMM description
        let amm = amm_info.amm;
        assert_eq!(amm.account, "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM");
        assert_eq!(amm.amount, Amount::XRPAmount("296890496".into()));
        assert_eq!(amm.trading_fee, 600);
        assert_eq!(amm.asset2_frozen, Some(false));

        // Verify auction slot
        let auction_slot = amm.auction_slot.unwrap();
        assert_eq!(auction_slot.account, "rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm");
        assert_eq!(auction_slot.discounted_fee, 60);
        assert_eq!(auction_slot.time_interval, 0);

        // Verify auth accounts
        let auth_accounts = auction_slot.auth_accounts.unwrap();
        assert_eq!(auth_accounts.len(), 2);
        assert_eq!(
            auth_accounts[0].account,
            "r3f2WpQMsAd8k4Zoijv2PZ78EYFJ2EdvgV"
        );
        assert_eq!(
            auth_accounts[1].account,
            "rnW8FAPgpQgA6VoESnVrUVJHBdq9QAtRZs"
        );

        // Verify vote slots
        let vote_slots = amm.vote_slots.unwrap();
        assert_eq!(vote_slots.len(), 1);
        assert_eq!(vote_slots[0].account, "rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm");
        assert_eq!(vote_slots[0].trading_fee, 600);
        assert_eq!(vote_slots[0].vote_weight, 9684);
    }
}
