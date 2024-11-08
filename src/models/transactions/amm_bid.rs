use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{
    transactions::TransactionType, Currency, FlagCollection, IssuedCurrencyAmount, Model, NoFlags,
    XRPAmount,
};

use super::{AuthAccount, CommonFields, Memo, Signer, Transaction};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMBid<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub asset: Currency<'a>,
    pub asset2: Currency<'a>,
    pub bid_min: Option<IssuedCurrencyAmount<'a>>,
    pub bid_max: Option<IssuedCurrencyAmount<'a>>,
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

    fn get_transaction_type(&self) -> TransactionType {
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
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        bid_min: Option<IssuedCurrencyAmount<'a>>,
        bid_max: Option<IssuedCurrencyAmount<'a>>,
        auth_accounts: Option<Vec<AuthAccount>>,
    ) -> AMMBid<'a> {
        AMMBid {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::AMMBid,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            asset,
            asset2,
            bid_min,
            bid_max,
            auth_accounts,
        }
    }
}
