use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{
    Currency, FlagCollection, Model, NoFlags, XRPAmount, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

pub const AMM_VOTE_MAX_TRADING_FEE: u16 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMVote<'a> {
    pub common_fields: CommonFields<'a, NoFlags>,
    pub asset: Currency<'a>,
    pub asset2: Currency<'a>,
    pub trading_fee: Option<u16>,
}

impl Model for AMMVote<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
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

    fn get_transaction_type(&self) -> super::TransactionType {
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
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        trading_fee: Option<u16>,
    ) -> AMMVote<'a> {
        AMMVote {
            common_fields: CommonFields {
                account,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                transaction_type: TransactionType::AMMVote,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            asset,
            asset2,
            trading_fee,
        }
    }
}
