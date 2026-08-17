use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies, XRPLModelException,
};
use crate::models::{FlagCollection, NoFlags};

use super::confidential_mpt_constants::{
    validate_hex_length, CIPHERTEXT_LENGTH, COMMITMENT_LENGTH, SEND_PROOF_LENGTH,
};
use super::mptoken_issuance_set::validate_mptoken_issuance_id;
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

    /// Arbitrary tag that identifies the reason for the transfer, or a hosted
    /// recipient at the destination account.
    pub destination_tag: Option<u32>,

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
        self._get_destination_error()?;
        self._get_field_length_errors()?;
        self.validate_currencies()
    }
}

impl<'a> ConfidentialMPTSend<'a> {
    /// rippled rejects a self-send with `temMALFORMED`.
    fn _get_destination_error(&self) -> crate::models::XRPLModelResult<()> {
        if self.destination == self.common_fields.account {
            return Err(XRPLModelException::ValueEqualsValue {
                field1: "destination".into(),
                field2: "account".into(),
            });
        }
        Ok(())
    }

    /// Ciphertext, commitment and proof lengths (`temBAD_CIPHERTEXT` /
    /// `temMALFORMED` in rippled's preflight).
    fn _get_field_length_errors(&self) -> crate::models::XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        validate_hex_length(
            "sender_encrypted_amount",
            self.sender_encrypted_amount.as_ref(),
            CIPHERTEXT_LENGTH,
        )?;
        validate_hex_length(
            "destination_encrypted_amount",
            self.destination_encrypted_amount.as_ref(),
            CIPHERTEXT_LENGTH,
        )?;
        validate_hex_length(
            "issuer_encrypted_amount",
            self.issuer_encrypted_amount.as_ref(),
            CIPHERTEXT_LENGTH,
        )?;
        if let Some(auditor) = self.auditor_encrypted_amount.as_deref() {
            validate_hex_length("auditor_encrypted_amount", auditor, CIPHERTEXT_LENGTH)?;
        }
        validate_hex_length(
            "amount_commitment",
            self.amount_commitment.as_ref(),
            COMMITMENT_LENGTH,
        )?;
        validate_hex_length(
            "balance_commitment",
            self.balance_commitment.as_ref(),
            COMMITMENT_LENGTH,
        )?;
        validate_hex_length("zk_proof", self.zk_proof.as_ref(), SEND_PROOF_LENGTH)
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
        destination_tag: Option<u32>,
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
            destination_tag,
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
            destination_tag: None,
            mptoken_issuance_id: "610F33".repeat(8).into(),
            sender_encrypted_amount: "AD".repeat(66).into(),
            destination_encrypted_amount: "DF".repeat(66).into(),
            issuer_encrypted_amount: "BC".repeat(66).into(),
            amount_commitment: "04".repeat(33).into(),
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

    #[test]
    fn test_new_builder_and_accessors() {
        let mut tx = ConfidentialMPTSend::new(
            "rSenderAccount11111111111111111".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "rRecipientAccount111111111111".into(),
            None,
            "610F33".repeat(8).into(),
            "AD".repeat(66).into(),
            "DF".repeat(66).into(),
            "BC".repeat(66).into(),
            "04".repeat(33).into(),
            "03".repeat(33).into(),
            "84".repeat(946).into(),
            None,
            None,
        )
        .with_fee(XRPAmount::from("15000"))
        .with_sequence(9);

        assert_eq!(tx.get_common_fields().sequence, Some(9));
        assert_eq!(tx.get_common_fields().fee, Some(XRPAmount::from("15000")));
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::ConfidentialMPTSend
        );
        assert!(tx.get_errors().is_ok());

        let common =
            <ConfidentialMPTSend as Transaction<'_, NoFlags>>::get_mut_common_fields(&mut tx);
        assert_eq!(common.sequence, Some(9));
    }
}
