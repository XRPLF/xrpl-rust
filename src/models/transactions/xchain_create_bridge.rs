use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::exceptions::XRPLXChainCreateBridgeException, Amount, FlagCollection, Model,
    NoFlags, XChainBridge, XRPAmount, XRPLModelResult, XRP,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCreateBridge<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub signature_reward: Amount<'a>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    pub min_account_create_amount: Option<XRPAmount<'a>>,
}

impl Model for XChainCreateBridge<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_same_door_error()?;
        self.get_account_door_mismatch_error()?;
        self.get_cross_currency_bridge_not_allowed_error()?;
        self.get_min_account_create_amount_for_iou_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainCreateBridge<'a> {
    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }
}

impl<'a> XChainCreateBridge<'a> {
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
        signature_reward: Amount<'a>,
        xchain_bridge: XChainBridge<'a>,
        min_account_create_amount: Option<XRPAmount<'a>>,
    ) -> XChainCreateBridge<'a> {
        XChainCreateBridge {
            common_fields: CommonFields::new(
                account,
                TransactionType::XChainCreateBridge,
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
            signature_reward,
            xchain_bridge,
            min_account_create_amount,
        }
    }

    fn get_same_door_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if bridge.issuing_chain_door == bridge.locking_chain_door {
            Err(XRPLXChainCreateBridgeException::SameDoorAccounts.into())
        } else {
            Ok(())
        }
    }

    fn get_account_door_mismatch_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if [&bridge.issuing_chain_door, &bridge.locking_chain_door]
            .contains(&&self.common_fields.account)
        {
            Err(XRPLXChainCreateBridgeException::AccountDoorMismatch.into())
        } else {
            Ok(())
        }
    }

    fn get_cross_currency_bridge_not_allowed_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if (bridge.locking_chain_issue == XRP::new().into())
            != (bridge.issuing_chain_issue == XRP::new().into())
        {
            Err(XRPLXChainCreateBridgeException::CrossCurrencyBridgeNotAllowed.into())
        } else {
            Ok(())
        }
    }

    fn get_min_account_create_amount_for_iou_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if self.min_account_create_amount.is_some()
            && bridge.locking_chain_issue != XRP::new().into()
        {
            Err(XRPLXChainCreateBridgeException::MinAccountCreateAmountForIOU.into())
        } else {
            Ok(())
        }
    }
}
