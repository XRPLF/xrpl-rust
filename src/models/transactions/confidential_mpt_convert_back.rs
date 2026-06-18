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

/// A `ConfidentialMPTConvertBack` transaction converts confidential MPT
/// value back to public form (XLS-0096 §10). The withdrawal amount is
/// revealed plaintext; the holder proves it doesn't exceed their balance
/// without revealing the balance itself.
///
/// The 816-byte `ZKProof` field carries:
///   - 128 B compact AND-composed sigma (balance ownership + key linkage)
///   - 688 B single Bulletproof (remainder is non-negative)
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
pub struct ConfidentialMPTConvertBack<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,

    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,

    /// Plaintext withdrawal amount (revealed publicly).
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext to be subtracted from holder's `CB_S`.
    pub holder_encrypted_amount: Cow<'a, str>,

    /// 66-byte ElGamal ciphertext to be subtracted from issuer mirror.
    pub issuer_encrypted_amount: Cow<'a, str>,

    /// 32-byte ElGamal randomness `r`. Revealed for deterministic
    /// verification of the ciphertexts above.
    pub blinding_factor: Cow<'a, str>,

    /// 33-byte Pedersen commitment to the holder's current balance.
    pub balance_commitment: Cow<'a, str>,

    /// 816-byte composite proof.
    #[serde(rename = "ZKProof")]
    pub zk_proof: Cow<'a, str>,

    /// 66-byte ciphertext for the auditor mirror. Required iff the
    /// issuance has an `AuditorEncryptionKey` registered.
    pub auditor_encrypted_amount: Option<Cow<'a, str>>,
}

impl<'a> Model for ConfidentialMPTConvertBack<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for ConfidentialMPTConvertBack<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for ConfidentialMPTConvertBack<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> ConfidentialMPTConvertBack<'a> {
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
        mptoken_issuance_id: Cow<'a, str>,
        mpt_amount: Cow<'a, str>,
        holder_encrypted_amount: Cow<'a, str>,
        issuer_encrypted_amount: Cow<'a, str>,
        blinding_factor: Cow<'a, str>,
        balance_commitment: Cow<'a, str>,
        zk_proof: Cow<'a, str>,
        auditor_encrypted_amount: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::ConfidentialMPTConvertBack,
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
            mptoken_issuance_id,
            mpt_amount,
            holder_encrypted_amount,
            issuer_encrypted_amount,
            blinding_factor,
            balance_commitment,
            zk_proof,
            auditor_encrypted_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let tx = ConfidentialMPTConvertBack {
            common_fields: CommonFields {
                account: "rUserAccount11111111111111111111".into(),
                transaction_type: TransactionType::ConfidentialMPTConvertBack,
                ..Default::default()
            },
            mptoken_issuance_id: "610F33".repeat(4).into(),
            mpt_amount: "500".into(),
            holder_encrypted_amount: "AD".repeat(66).into(),
            issuer_encrypted_amount: "BC".repeat(66).into(),
            blinding_factor: "12".repeat(32).into(),
            balance_commitment: "03".repeat(33).into(),
            zk_proof: "AB".repeat(816).into(),
            auditor_encrypted_amount: None,
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"TransactionType\":\"ConfidentialMPTConvertBack\""));
        assert!(json.contains("\"BalanceCommitment\""));

        let round_tripped: ConfidentialMPTConvertBack = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, tx);
    }
}
