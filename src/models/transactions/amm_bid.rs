use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::TransactionType, Currency, FlagCollection, IssuedCurrencyAmount, Model, NoFlags,
    XRPAmount,
};

use super::{AuthAccount, CommonFields, Memo, Signer, Transaction};

/// Bid on an Automated Market Maker's (AMM's) auction slot.
///
/// If you win, you can trade against the AMM at a discounted fee until you are outbid
/// or 24 hours have passed.
/// If you are outbid before 24 hours have passed, you are refunded part of the cost
/// of your bid based on how much time remains.
/// You bid using the AMM's LP Tokens; the amount of a winning bid is returned
/// to the AMM, decreasing the outstanding balance of LP Tokens.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AMMBid<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// Pay at least this LPToken amount for the slot.
    /// Setting this value higher makes it harder for others to outbid you.
    /// If omitted, pay the minimum necessary to win the bid.
    pub bid_min: Option<IssuedCurrencyAmount<'a>>,
    /// Pay at most this LPToken amount for the slot.
    /// If the cost to win the bid is higher than this amount, the transaction fails.
    /// If omitted, pay as much as necessary to win the bid.
    pub bid_max: Option<IssuedCurrencyAmount<'a>>,
    /// A list of up to 4 additional accounts that you allow to trade at the discounted fee.
    /// This cannot include the address of the transaction sender.
    pub auth_accounts: Option<Vec<AuthAccount>>,
}

impl Model for AMMBid<'_> {}

impl<'a> Transaction<'a, NoFlags> for AMMBid<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> AMMBid<'_> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        bid_min: Option<IssuedCurrencyAmount<'a>>,
        bid_max: Option<IssuedCurrencyAmount<'a>>,
        auth_accounts: Option<Vec<AuthAccount>>,
    ) -> AMMBid<'a> {
        AMMBid {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMBid,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            asset,
            asset2,
            bid_min,
            bid_max,
            auth_accounts,
        }
    }
}
