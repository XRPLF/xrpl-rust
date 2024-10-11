use alloc::{borrow::Cow, vec::Vec};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    models::{
        transactions::exceptions::XRPLXChainClaimException, Amount, Currency, FlagCollection,
        Model, NoFlags, XChainBridge,
    },
    Err,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[skip_serializing_none]
pub struct XChainClaim<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub destination: Cow<'a, str>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    pub destination_tag: Option<u32>,
}

impl Model for XChainClaim<'_> {
    fn get_errors(&self) -> Result<()> {
        self.get_amount_mismatch_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainClaim<'a> {
    fn get_transaction_type(&self) -> super::TransactionType {
        TransactionType::XChainClaim
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }
}

impl<'a> XChainClaim<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<crate::models::XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        destination: Cow<'a, str>,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        destination_tag: Option<u32>,
    ) -> XChainClaim<'a> {
        XChainClaim {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainClaim,
                account_txn_id,
                fee,
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                flags: FlagCollection::default(),
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            amount,
            destination,
            xchain_bridge,
            xchain_claim_id,
            destination_tag,
        }
    }

    fn get_amount_mismatch_error(&self) -> Result<()> {
        let bridge = &self.xchain_bridge;
        match &self.amount {
            Amount::XRPAmount(amount) => {
                if Currency::from(amount) != bridge.locking_chain_issue
                    && Currency::from(amount) != bridge.issuing_chain_issue
                {
                    Err!(XRPLXChainClaimException::AmountMismatch)
                } else {
                    Ok(())
                }
            }
            Amount::IssuedCurrencyAmount(amount) => {
                if Currency::from(amount) != bridge.locking_chain_issue
                    && Currency::from(amount) != bridge.issuing_chain_issue
                {
                    Err!(XRPLXChainClaimException::AmountMismatch)
                } else {
                    Ok(())
                }
            }
        }
    }
}
