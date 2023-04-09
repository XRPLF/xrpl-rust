use crate::models::ledger::LedgerEntryType;
use crate::models::{amount::Amount, Currency, Model};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct AuthAccount<'a> {
        pub account: Cow<'a, str>,
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new, Default)]
#[serde(rename_all = "PascalCase")]
/// `<https://xrpl.org/amm.html#auction-slot-object>`
pub struct AuctionSlot<'a> {
    /// The current owner of this auction slot.
    pub account: Cow<'a, str>,
    /// The trading fee to be charged to the auction owner, in the same format as TradingFee. By
    /// default this is 0, meaning that the auction owner can trade at no fee instead of the
    /// standard fee for this AMM.
    pub discounted_fee: u32,
    /// The time when this slot expires, in seconds since the Ripple Epoch.
    pub expiration: u32,
    /// The amount the auction owner paid to win this slot, in LP Tokens.
    pub price: Amount<'a>,
    /// A list of at most 4 additional accounts that are authorized to trade at the discounted fee
    /// for this AMM instance.
    #[serde(borrow = "'a")]
    pub auth_accounts: Option<Vec<AuthAccount<'a>>>,
}

serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct VoteEntry<'a> {
        pub account: Cow<'a, str>,
        pub trading_fee: u16,
        pub vote_weight: u32,
    }
}

/// The `AMM` object type describes a single Automated Market Maker (`AMM`) instance.
///
/// `<https://xrpl.org/amm.html#amm-fields>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AMM<'a> {
    /// The value `0x0079`, mapped to the string `AMM`, indicates that this is an `AMM` object.
    pub ledger_entry_type: LedgerEntryType,
    /// Currently there are no flags for the AMM ledger object
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The address of the special account that holds this `AMM's` assets.
    #[serde(rename = "AMMAccount")]
    pub amm_account: Cow<'a, str>,
    /// The definition for one of the two assets this `AMM` holds. In JSON, this is an object with
    /// `currency` and `issuer` fields.
    pub asset: Currency<'a>,
    /// The definition for the other asset this `AMM` holds. In JSON, this is an object with
    /// `currency` and `issuer` fields.
    pub asset2: Currency<'a>,
    /// The total outstanding balance of liquidity provider tokens from this `AMM` instance.
    /// The holders of these tokens can vote on the `AMM's` trading fee in proportion to their
    /// holdings, or redeem the tokens for a share of the `AMM's` assets which grows with the
    /// trading fees collected.
    #[serde(rename = "LPTokenBalance")]
    pub lptoken_balance: Amount<'a>,
    /// The percentage fee to be charged for trades against this `AMM` instance,
    /// in units of 1/100,000. The maximum value is 1000, for a 1% fee.
    pub trading_fee: u16,
    /// Details of the current owner of the auction slot, as an `AuctionSlot` object.
    #[serde(borrow = "'a")]
    pub auction_slot: Option<AuctionSlot<'a>>,
    /// A list of vote objects, representing votes on the pool's trading fee.
    pub vote_slots: Option<Vec<VoteEntry<'a>>>,
}

impl<'a> Default for AMM<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::AMM,
            flags: Default::default(),
            index: Default::default(),
            amm_account: Default::default(),
            asset: Default::default(),
            asset2: Default::default(),
            lptoken_balance: Default::default(),
            trading_fee: Default::default(),
            auction_slot: Default::default(),
            vote_slots: Default::default(),
        }
    }
}

impl<'a> Model for AMM<'a> {}

impl<'a> AMM<'a> {
    pub fn new(
        index: Cow<'a, str>,
        amm_account: Cow<'a, str>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        lptoken_balance: Amount<'a>,
        trading_fee: u16,
        auction_slot: Option<AuctionSlot<'a>>,
        vote_slots: Option<Vec<VoteEntry<'a>>>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::AMM,
            flags: 0,
            index,
            amm_account,
            asset,
            asset2,
            lptoken_balance,
            trading_fee,
            auction_slot,
            vote_slots,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::amount::{Amount, IssuedCurrencyAmount};
    use crate::models::currency::{Currency, IssuedCurrency, XRP};
    use crate::models::ledger::amm::{AuctionSlot, AuthAccount, VoteEntry, AMM};
    use alloc::borrow::Cow;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let amm = AMM::new(
            Cow::from("ForTest"),
            Cow::from("rE54zDvgnghAoPopCgvtiqWNq3dU5y836S"),
            Currency::XRP(XRP::new()),
            Currency::IssuedCurrency(IssuedCurrency::new(
                "TST".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
                "71150.53584131501".into(),
            )),
            600,
            Some(AuctionSlot::new(
                Cow::from("rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm"),
                0,
                721870180,
                Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                    "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
                    "0.8696263565463045".into(),
                )),
                Some(vec![
                    AuthAccount::new(Cow::from("rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg")),
                    AuthAccount::new(Cow::from("rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv")),
                ]),
            )),
            Some(vec![VoteEntry::new(
                Cow::from("rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm"),
                600,
                100000,
            )]),
        );
        let amm_json = serde_json::to_string(&amm).unwrap();
        let actual = amm_json.as_str();
        let expected = r#"{"LedgerEntryType":"AMM","Flags":0,"index":"ForTest","AMMAccount":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","Asset":{"currency":"XRP"},"Asset2":{"currency":"TST","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"},"LPTokenBalance":{"currency":"039C99CD9AB0B70B32ECDA51EAAE471625608EA2","issuer":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","value":"71150.53584131501"},"TradingFee":600,"AuctionSlot":{"Account":"rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm","DiscountedFee":0,"Expiration":721870180,"Price":{"currency":"039C99CD9AB0B70B32ECDA51EAAE471625608EA2","issuer":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","value":"0.8696263565463045"},"AuthAccounts":[{"AuthAccount":{"Account":"rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg"}},{"AuthAccount":{"Account":"rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv"}}]},"VoteSlots":[{"VoteEntry":{"Account":"rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm","TradingFee":600,"VoteWeight":100000}}]}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
