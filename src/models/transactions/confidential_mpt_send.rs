use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// A `ConfidentialMPTSend` transaction transfers a confidential MPT amount
/// from sender to destination, hiding the amount under EC-ElGamal
/// encryption (XLS-0096 §8). The amount is decrypted only by the recipient
/// (and the issuer / optional auditor via their mirror keys).
///
/// The 946-byte `ZKProof` field carries:
///   - 192 B compact AND-composed sigma proof (ciphertext consistency,
///     Pedersen amount linkage, balance ownership)
///   - 754 B aggregated Bulletproof (range proof on amount AND remainder)
///
/// `CredentialIDs` (XLS-70) are honored when the destination requires
/// pre-authorization.
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct ConfidentialMPTSend<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    /// Destination XRPL account.
    pub destination: Cow<'a, str>,

    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext debited from the sender's `CB_S`.
    pub sender_encrypted_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext credited to the receiver's `CB_IN`.
    pub destination_encrypted_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext used to update both the sender's and
    /// receiver's `IssuerEncryptedBalance` mirrors.
    pub issuer_encrypted_amount: Cow<'a, str>,

    /// 33-byte Pedersen commitment to the transfer amount.
    pub amount_commitment: Cow<'a, str>,

    /// 33-byte Pedersen commitment to the sender's confidential balance.
    pub balance_commitment: Cow<'a, str>,

    /// 946-byte composite ZK proof (192 B compact sigma + 754 B aggregated
    /// Bulletproof).
    #[serde(rename = "ZKProof")]
    pub zk_proof: Cow<'a, str>,

    /// 66-byte ciphertext for the auditor mirror. Required iff the
    /// issuance has an `AuditorEncryptionKey` registered.
    pub auditor_encrypted_amount: Option<Cow<'a, str>>,

    /// XLS-70 credentials presented to satisfy the destination's
    /// `DepositPreauth` / `AuthorizeCredentials` requirement, if any.
    #[serde(rename = "CredentialIDs")]
    pub credential_ids: Option<Vec<Cow<'a, str>>>,
}

impl<'a> Model for ConfidentialMPTSend<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTSend<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTSend<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTSend<'a> {
    #[allow(clippy::too_many_arguments)]
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
        destination: Cow<'a, str>,
        mptoken_issuance_id: Cow<'a, str>,
        sender_encrypted_amount: Cow<'a, str>,
        destination_encrypted_amount: Cow<'a, str>,
        issuer_encrypted_amount: Cow<'a, str>,
        amount_commitment: Cow<'a, str>,
        balance_commitment: Cow<'a, str>,
        zk_proof: Cow<'a, str>,
        auditor_encrypted_amount: Option<Cow<'a, str>>,
        credential_ids: Option<Vec<Cow<'a, str>>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTSend,
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
            destination,
            mptoken_issuance_id,
            sender_encrypted_amount,
            destination_encrypted_amount,
            issuer_encrypted_amount,
            amount_commitment,
            balance_commitment,
            zk_proof,
            auditor_encrypted_amount,
            credential_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let tx = ConfidentialMPTSend {
            common_fields: CommonFields {
                account: "rSenderAccount11111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTSend,
                ..Default::default()
            },
            destination: "rRecipientAccount111111111111".into(),
            mptoken_issuance_id: "610F33".repeat(4).into(),
            sender_encrypted_amount:      "AD".repeat(66).into(),
            destination_encrypted_amount: "DF".repeat(66).into(),
            issuer_encrypted_amount:      "BC".repeat(66).into(),
            amount_commitment:  "04".repeat(33).into(),
            balance_commitment: "03".repeat(33).into(),
            zk_proof: "84".repeat(946).into(),
            auditor_encrypted_amount: None,
            credential_ids: None,
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTSend\""));
        assert!(json.contains("\"Destination\":\"rRecipientAccount"));
        assert!(json.contains("\"AmountCommitment\""));
        assert!(json.contains("\"BalanceCommitment\""));
        assert!(json.contains("\"ZKProof\""));

        let round_tripped: ConfidentialMPTSend = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }
}
