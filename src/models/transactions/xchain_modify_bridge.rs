use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::exceptions::XRPLXChainModifyBridgeException, Amount, FlagCollection, Model,
    XChainBridge, XRPAmount, XRPLModelResult, XRP,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum XChainModifyBridgeFlags {
    /// Clears the MinAccountCreateAmount of the bridge.
    TfClearAccountCreateAmount = 0x00010000,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainModifyBridge<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, XChainModifyBridgeFlags>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    pub min_account_create_amount: Option<Amount<'a>>,
    pub signature_reward: Option<Amount<'a>>,
}

impl Model for XChainModifyBridge<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_must_change_or_clear_error()?;
        self.get_account_door_mismatch_error()?;
        self.get_cannot_have_min_account_create_amount()?;
        Ok(())
    }
}

impl<'a> Transaction<'a, XChainModifyBridgeFlags> for XChainModifyBridge<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, XChainModifyBridgeFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, XChainModifyBridgeFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> XChainModifyBridge<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<XChainModifyBridgeFlags>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        xchain_bridge: XChainBridge<'a>,
        min_account_create_amount: Option<Amount<'a>>,
        signature_reward: Option<Amount<'a>>,
    ) -> XChainModifyBridge<'a> {
        XChainModifyBridge {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainModifyBridge,
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
            },
            xchain_bridge,
            min_account_create_amount,
            signature_reward,
        }
    }

    fn get_must_change_or_clear_error(&self) -> XRPLModelResult<()> {
        if self.signature_reward.is_none()
            && self.min_account_create_amount.is_none()
            && !self.has_flag(&XChainModifyBridgeFlags::TfClearAccountCreateAmount)
        {
            Err(XRPLXChainModifyBridgeException::MustChangeOrClear.into())
        } else {
            Ok(())
        }
    }

    fn get_account_door_mismatch_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if [&bridge.locking_chain_door, &bridge.issuing_chain_door]
            .contains(&&self.get_common_fields().account)
        {
            Err(XRPLXChainModifyBridgeException::AccountDoorMismatch.into())
        } else {
            Ok(())
        }
    }

    fn get_cannot_have_min_account_create_amount(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        if self.min_account_create_amount.is_some()
            && bridge.locking_chain_issue != XRP::new().into()
        {
            Err(XRPLXChainModifyBridgeException::CannotHaveMinAccountCreateAmount.into())
        } else {
            Ok(())
        }
    }
}
