use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    core::addresscodec::is_valid_classic_address,
    models::{
        transactions::exceptions::XRPLXChainCreateClaimIDException, FlagCollection, Model, NoFlags,
        ValidateCurrencies, XChainBridge, XRPAmount, XRPLModelResult,
    },
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, xrpl_rust_macros::ValidateCurrencies)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCreateClaimID<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub other_chain_source: Cow<'a, str>,
    pub signature_reward: XRPAmount<'a>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
}

impl Model for XChainCreateClaimID<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;
        self.get_other_chain_source_is_invalid_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainCreateClaimID<'a> {
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

impl<'a> XChainCreateClaimID<'a> {
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
        other_chain_source: Cow<'a, str>,
        signature_reward: XRPAmount<'a>,
        xchain_bridge: XChainBridge<'a>,
    ) -> XChainCreateClaimID<'a> {
        XChainCreateClaimID {
            common_fields: CommonFields::new(
                account,
                TransactionType::XChainCreateClaimID,
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
            other_chain_source,
            signature_reward,
            xchain_bridge,
        }
    }

    fn get_other_chain_source_is_invalid_error(&self) -> XRPLModelResult<()> {
        if !is_valid_classic_address(self.other_chain_source.as_ref()) {
            Err(XRPLXChainCreateClaimIDException::OtherChainSourceIsInvalid.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test_xchain_create_claim_id {
    use super::XChainCreateClaimID;
    use crate::models::{Model, XChainBridge, XRP};
    use alloc::borrow::Cow;

    const ACCOUNT: &str = "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ";
    const ACCOUNT2: &str = "rpZc4mVfWUif9CRoHRKKcmhu1nx2xktxBo";
    const ISSUER: &str = "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf";
    const GENESIS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    const SOURCE: &str = "rJrRMgiRgrU6hDF4pgu5DXQdWyPbY35ErN";
    const SIGNATURE_REWARD: &str = "200";

    fn xrp_bridge<'a>() -> XChainBridge<'a> {
        XChainBridge {
            locking_chain_door: Cow::Borrowed(ACCOUNT),
            locking_chain_issue: XRP::new().into(),
            issuing_chain_door: Cow::Borrowed(GENESIS),
            issuing_chain_issue: XRP::new().into(),
        }
    }

    #[test]
    fn test_successful() {
        let txn = XChainCreateClaimID::new(
            Cow::Borrowed(ACCOUNT),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Cow::Borrowed(SOURCE),
            Cow::Borrowed(SIGNATURE_REWARD).into(),
            xrp_bridge(),
        );
        assert!(txn.validate().is_ok());
    }

    #[test]
    #[should_panic]
    fn test_bad_signature_reward() {
        let txn = XChainCreateClaimID::new(
            Cow::Borrowed(ACCOUNT),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Cow::Borrowed(SOURCE),
            Cow::Borrowed("hello").into(),
            xrp_bridge(),
        );
        txn.validate().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_bad_other_chain_source() {
        let txn = XChainCreateClaimID::new(
            Cow::Borrowed(ACCOUNT),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Cow::Borrowed("hello"),
            Cow::Borrowed(SIGNATURE_REWARD).into(),
            xrp_bridge(),
        );
        txn.validate().unwrap();
    }
}
