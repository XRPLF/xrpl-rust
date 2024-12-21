use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::exceptions::XRPLXChainClaimException, Amount, Currency, FlagCollection, Model,
    NoFlags, XChainBridge, XRPLModelResult,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainClaim<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub destination: Cow<'a, str>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,
}

impl Model for XChainClaim<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_amount_mismatch_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainClaim<'a> {
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

    fn get_amount_mismatch_error(&self) -> XRPLModelResult<()> {
        let bridge = &self.xchain_bridge;
        match &self.amount {
            Amount::XRPAmount(amount) => {
                if Currency::from(amount) != bridge.locking_chain_issue
                    && Currency::from(amount) != bridge.issuing_chain_issue
                {
                    Err(XRPLXChainClaimException::AmountMismatch.into())
                } else {
                    Ok(())
                }
            }
            Amount::IssuedCurrencyAmount(amount) => {
                if Currency::from(amount) != bridge.locking_chain_issue
                    && Currency::from(amount) != bridge.issuing_chain_issue
                {
                    Err(XRPLXChainClaimException::AmountMismatch.into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "wallet")]
mod test_sign {
    use crate::{
        models::{
            transactions::xchain_claim::XChainClaim, IssuedCurrency, IssuedCurrencyAmount,
            XChainBridge, XRP,
        },
        transaction::sign,
        wallet::Wallet,
    };

    #[test]
    fn test_sign_xchain_claim_xrp() {
        let wallet = Wallet::new("sEdVWgwiHxBmFoMGJBoPZf6H1XSLLGd", 0).unwrap();
        let mut txn = XChainClaim::new(
            "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ".into(),
            None,
            Some("10".into()),
            None,
            None,
            Some(19048),
            None,
            None,
            None,
            "123456789".into(),
            "rJrRMgiRgrU6hDF4pgu5DXQdWyPbY35ErN".into(),
            XChainBridge::new(
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                XRP::new().into(),
                "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ".into(),
                XRP::new().into(),
            ),
            "3".into(),
            None,
        );
        sign(&mut txn, &wallet, false).unwrap();
        assert_eq!(
            txn.common_fields.txn_signature,
            Some(
                "7A079C6360AA6E5BDDFC149175E2E47FC5B561888D149C3097D41449601F9D\
                C07A1B5BFD69EAF5D16567076B61AADBFF3FCA243B1A8A492828FEA21CA8416E05"
                    .into()
            )
        );
    }

    #[test]
    fn test_sign_xchain_claim_iou() {
        let wallet = Wallet::new("sEdVWgwiHxBmFoMGJBoPZf6H1XSLLGd", 0).unwrap();
        let mut txn = XChainClaim::new(
            "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ".into(),
            None,
            Some("10".into()),
            None,
            None,
            Some(19048),
            None,
            None,
            None,
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf".into(),
                "123".into(),
            )
            .into(),
            "rJrRMgiRgrU6hDF4pgu5DXQdWyPbY35ErN".into(),
            XChainBridge::new(
                "rpZc4mVfWUif9CRoHRKKcmhu1nx2xktxBo".into(),
                IssuedCurrency::new("USD".into(), "rpZc4mVfWUif9CRoHRKKcmhu1nx2xktxBo".into())
                    .into(),
                "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ".into(),
                IssuedCurrency::new("USD".into(), "rGWrZyQqhTp9Xu7G5Pkayo7bXjH4k4QYpf".into())
                    .into(),
            ),
            "3".into(),
            None,
        );
        sign(&mut txn, &wallet, false).unwrap();
        assert_eq!(
            txn.common_fields.txn_signature,
            Some(
                "1A2136A8D8FA7176178596EE341C9FB950D728DFB18C71AECFE9E5426E9481\
                1C7E40F159FED49721B77137BE272B199ADED3DEF3D68DB7C24F32E39F2AAED40B"
                    .into()
            )
        );
    }
}
