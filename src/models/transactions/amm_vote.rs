use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    Currency, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

pub const AMM_VOTE_MAX_TRADING_FEE: u16 = 1000;

/// Vote on the trading fee for an Automated Market Maker (AMM) instance.
///
/// Up to 8 accounts can vote in proportion to the amount of the AMM's LP Tokens
/// they hold.
/// Each new vote re-calculates the AMM's trading fee based on a weighted average
/// of the votes.
#[skip_serializing_none]
#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, Clone, xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct AMMVote<'a> {
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// The proposed fee to vote for, in units of 1/100,000; a value of 1 is equivalent
    /// to 0.001%.
    /// The maximum value is 1000, indicating a 1% fee.
    pub trading_fee: Option<u16>,
}

impl Model for AMMVote<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;
        if let Some(trading_fee) = self.trading_fee {
            if trading_fee > AMM_VOTE_MAX_TRADING_FEE {
                return Err(XRPLModelException::ValueTooHigh {
                    field: "trading_fee".into(),
                    max: AMM_VOTE_MAX_TRADING_FEE.into(),
                    found: trading_fee.into(),
                });
            }
        }

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMVote<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> AMMVote<'a> {
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
        trading_fee: Option<u16>,
    ) -> AMMVote<'a> {
        AMMVote {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMVote,
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
            trading_fee,
        }
    }
}
