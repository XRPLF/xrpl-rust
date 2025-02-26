use crate::models::ledger::objects::LedgerEntryType;
use crate::models::FlagCollection;
use crate::models::NoFlags;
use crate::models::{amount::Amount, Currency, Model};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct AuthAccount {
        pub account: String,
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
    pub auth_accounts: Option<Vec<AuthAccount>>,
}

serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct VoteEntry {
        pub account: String,
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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the AMM model.
    //
    // See AMM fields:
    // `<https://xrpl.org/amm.html#amm-fields>`
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
    pub auction_slot: Option<AuctionSlot<'a>>,
    /// A list of vote objects, representing votes on the pool's trading fee.
    pub vote_slots: Option<Vec<VoteEntry>>,
}

impl<'a> Model for AMM<'a> {}

impl<'a> LedgerObject<NoFlags> for AMM<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> AMM<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        amm_account: Cow<'a, str>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        lptoken_balance: Amount<'a>,
        trading_fee: u16,
        auction_slot: Option<AuctionSlot<'a>>,
        vote_slots: Option<Vec<VoteEntry>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::AMM,
                index,
                ledger_index,
            },
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
    use crate::models::ledger::objects::amm::{AuctionSlot, AuthAccount, VoteEntry, AMM};
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let amm = AMM::new(
            Some(Cow::from("ForTest")),
            None,
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
                    AuthAccount::new("rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg".to_string()),
                    AuthAccount::new("rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv".to_string()),
                ]),
            )),
            Some(vec![VoteEntry::new(
                "rJVUeRqDFNs2xqA7ncVE6ZoAhPUoaJJSQm".to_string(),
                600,
                100000,
            )]),
        );
        let serialized = serde_json::to_string(&amm).unwrap();

        let deserialized: AMM = serde_json::from_str(&serialized).unwrap();

        assert_eq!(amm, deserialized);
    }

    // TODO: test_deserialize
}
