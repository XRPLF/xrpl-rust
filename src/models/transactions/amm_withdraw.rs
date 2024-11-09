use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{Amount, Currency, FlagCollection, IssuedCurrencyAmount, Model, XRPAmount};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum AMMWithdrawFlag {
    TfLpToken = 0x00010000,
    TfWithdrawAll = 0x00020000,
    TfOneAssetWithdrawAll = 0x00040000,
    TfSingleAsset = 0x00080000,
    TfTwoAsset = 0x00100000,
    TfOneAssetLpToken = 0x00200000,
    TfLimitLpToken = 0x00400000,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMWithdraw<'a> {
    pub common_fields: CommonFields<'a, AMMWithdrawFlag>,
    pub asset: Currency<'a>,
    pub asset2: Currency<'a>,
    pub amount: Option<Amount<'a>>,
    pub amount2: Option<Amount<'a>>,
    pub e_price: Option<Amount<'a>>,
    pub lp_token_in: Option<IssuedCurrencyAmount<'a>>,
}

impl Model for AMMWithdraw<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        if self.amount2.is_some() && self.amount.is_none() {
            Err(crate::models::XRPLModelException::FieldRequiresField {
                field1: "amount2".into(),
                field2: "amount".into(),
            })
        } else if self.e_price.is_some() && self.amount.is_none() {
            Err(crate::models::XRPLModelException::FieldRequiresField {
                field1: "e_price".into(),
                field2: "amount".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> Transaction<'a, AMMWithdrawFlag> for AMMWithdraw<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, AMMWithdrawFlag> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AMMWithdrawFlag> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> AMMWithdraw<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<AMMWithdrawFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        amount: Option<Amount<'a>>,
        amount2: Option<Amount<'a>>,
        e_price: Option<Amount<'a>>,
        lp_token_in: Option<IssuedCurrencyAmount<'a>>,
    ) -> Self {
        AMMWithdraw {
            common_fields: CommonFields {
                account,
                account_txn_id,
                fee,
                flags: flags.unwrap_or_default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
                transaction_type: TransactionType::AMMWithdraw,
            },
            asset,
            asset2,
            amount,
            amount2,
            e_price,
            lp_token_in,
        }
    }
}
