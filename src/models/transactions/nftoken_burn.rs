use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// Removes a NFToken object from the NFTokenPage in which it is being held,
/// effectively removing the token from the ledger (burning it).
///
/// See NFTokenBurn:
/// `<https://xrpl.org/nftokenburn.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenBurn<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::nftoken_burn")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the NFTokenBurn model.
    ///
    /// See NFTokenBurn fields:
    /// `<https://xrpl.org/nftokenburn.html#nftokenburn-fields>`
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    pub owner: Option<Cow<'a, str>>,
}

impl<'a> Default for NFTokenBurn<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::NFTokenBurn,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            nftoken_id: Default::default(),
            owner: Default::default(),
        }
    }
}

impl<'a> Model for NFTokenBurn<'a> {}

impl<'a> Transaction for NFTokenBurn<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> NFTokenBurn<'a> {
    pub fn new(
        account: Cow<'a, str>,
        nftoken_id: Cow<'a, str>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<Cow<'a, str>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        owner: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::NFTokenBurn,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags: None,
            memos,
            signers,
            nftoken_id,
            owner,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = NFTokenBurn::new(
            "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2".into(),
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            Some("10".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into()),
        );
        let default_json = r#"{"TransactionType":"NFTokenBurn","Account":"rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2","Fee":"10","NFTokenID":"000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65","Owner":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = NFTokenBurn::new(
            "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2".into(),
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            Some("10".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into()),
        );
        let default_json = r#"{"TransactionType":"NFTokenBurn","Account":"rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2","Owner":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B","Fee":"10","NFTokenID":"000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"}"#;

        let txn_as_obj: NFTokenBurn = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
