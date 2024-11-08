use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{Amount, FlagCollection, Model, NoFlags, XRPAmount, XRPLModelResult};

use super::{
    exceptions::{XRPLAMMCreateException, XRPLTransactionException},
    CommonFields, Memo, Signer, Transaction, TransactionType,
};

pub const AMM_CREATE_MAX_FEE: u16 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMCreate<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub amount2: Amount<'a>,
    pub trading_fee: u16,
}

impl Model for AMMCreate<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_tranding_fee_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMCreate<'a> {
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

impl<'a> AMMCreate<'a> {
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
        amount: Amount<'a>,
        amount2: Amount<'a>,
        trading_fee: u16,
    ) -> AMMCreate<'a> {
        AMMCreate {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::AMMCreate,
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
            amount,
            amount2,
            trading_fee,
        }
    }

    fn get_tranding_fee_error(&self) -> XRPLModelResult<()> {
        if self.trading_fee > AMM_CREATE_MAX_FEE {
            Err(
                XRPLTransactionException::from(XRPLAMMCreateException::TradingFeeOutOfRange {
                    max: AMM_CREATE_MAX_FEE,
                    found: self.trading_fee,
                })
                .into(),
            )
        } else {
            Ok(())
        }
    }
}
