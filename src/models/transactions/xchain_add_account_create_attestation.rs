use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, XChainBridge, XRPAmount};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAddAccountCreateAttestation<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub attestation_reward_account: Cow<'a, str>,
    pub attestation_signer_account: Cow<'a, str>,
    pub destination: Cow<'a, str>,
    pub other_chain_source: Cow<'a, str>,
    pub public_key: Cow<'a, str>,
    pub signature: Cow<'a, str>,
    pub signature_reward: Amount<'a>,
    pub was_locking_chain_send: u8,
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: Cow<'a, str>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
}

impl Model for XChainAddAccountCreateAttestation<'_> {}

impl<'a> Transaction<'a, NoFlags> for XChainAddAccountCreateAttestation<'a> {
    fn get_transaction_type(&self) -> super::TransactionType {
        TransactionType::XChainAddAccountCreateAttestation
    }

    fn get_common_fields(&self) -> &super::CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut super::CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }
}

impl<'a> XChainAddAccountCreateAttestation<'a> {
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
        attestation_reward_account: Cow<'a, str>,
        attestation_signer_account: Cow<'a, str>,
        destination: Cow<'a, str>,
        other_chain_source: Cow<'a, str>,
        public_key: Cow<'a, str>,
        signature: Cow<'a, str>,
        signature_reward: Amount<'a>,
        was_locking_chain_send: u8,
        xchain_account_create_count: Cow<'a, str>,
        xchain_bridge: XChainBridge<'a>,
    ) -> XChainAddAccountCreateAttestation<'a> {
        XChainAddAccountCreateAttestation {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainAddAccountCreateAttestation,
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
            destination,
            other_chain_source,
            public_key,
            signature,
            signature_reward,
            was_locking_chain_send,
            xchain_account_create_count,
            xchain_bridge,
        }
    }
}

#[cfg(test)]
mod test_serde {
    const EXAMPLE_JSON: &str = r#"{
        "Account": "rDr5okqGKmMpn44Bbhe5WAfDQx8e9XquEv",
        "Flags": 0,
        "TransactionType": "XChainAddAccountCreateAttestation",
        "OtherChainSource": "rUzB7yg1LcFa7m3q1hfrjr5w53vcWzNh3U",
        "Destination": "rJMfWNVbyjcCtds8kpoEjEbYQ41J5B6MUd",
        "Amount": "2000000000",
        "PublicKey": "EDF7C3F9C80C102AF6D241752B37356E91ED454F26A35C567CF6F8477960F66614",
        "Signature": "F95675BA8FDA21030DE1B687937A79E8491CE51832D6BEEBC071484FA5AF5B8A0E9AFF11A4AA46F09ECFFB04C6A8DAE8284AF3ED8128C7D0046D842448478500",
        "WasLockingChainSend": 1,
        "AttestationRewardAccount": "rpFp36UHW6FpEcZjZqq5jSJWY6UCj3k4Es",
        "AttestationSignerAccount": "rpWLegmW9WrFBzHUj7brhQNZzrxgLj9oxw",
        "XChainAccountCreateCount": "2",
        "SignatureReward": "204",
        "XChainBridge": {
            "LockingChainDoor": "r3nCVTbZGGYoWvZ58BcxDmiMUU7ChMa1eC",
            "LockingChainIssue": {
                "currency": "XRP"
            },
            "IssuingChainDoor": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "IssuingChainIssue": {
                "currency": "XRP"
            }
        },
        "Fee": "20"
    }"#;
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_deserialize() {
        let json = EXAMPLE_JSON;
        let deserialized: Result<XChainAddAccountCreateAttestation, _> = serde_json::from_str(json);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_serialize() {
        let attestation: XChainAddAccountCreateAttestation<'_> =
            serde_json::from_str(EXAMPLE_JSON).unwrap();
        let actual = serde_json::to_value(&attestation).unwrap();
        let expected: Value = serde_json::from_str(EXAMPLE_JSON).unwrap();

        assert_eq!(actual, expected);
    }
}
