use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, XChainBridge};

use super::{CommonFields, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAddClaimAttestation<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub attestation_reward_account: Cow<'a, str>,
    pub attestation_signer_account: Cow<'a, str>,
    pub other_chain_source: Cow<'a, str>,
    pub public_key: Cow<'a, str>,
    pub signature: Cow<'a, str>,
    pub was_locking_chain_send: u8,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    pub destination: Option<Cow<'a, str>>,
}

impl Model for XChainAddClaimAttestation<'_> {}

impl<'a> Transaction<'a, NoFlags> for XChainAddClaimAttestation<'a> {
    fn get_transaction_type(&self) -> super::TransactionType {
        TransactionType::XChainAddClaimAttestation
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }
}

impl<'a> XChainAddClaimAttestation<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<crate::models::XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<super::Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<super::Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        attestation_reward_account: Cow<'a, str>,
        attestation_signer_account: Cow<'a, str>,
        other_chain_source: Cow<'a, str>,
        public_key: Cow<'a, str>,
        signature: Cow<'a, str>,
        was_locking_chain_send: u8,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        destination: Option<Cow<'a, str>>,
    ) -> XChainAddClaimAttestation<'a> {
        XChainAddClaimAttestation {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainAddClaimAttestation,
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
            attestation_reward_account,
            attestation_signer_account,
            other_chain_source,
            public_key,
            signature,
            was_locking_chain_send,
            xchain_bridge,
            xchain_claim_id,
            destination,
        }
    }
}
