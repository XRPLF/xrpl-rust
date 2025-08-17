use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, NoFlags};
use crate::models::{
    Model, ValidateCurrencies,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Removes a NFToken object from the NFTokenPage in which it is being held,
/// effectively removing the token from the ledger (burning it).
///
/// See NFTokenBurn:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokenburn>`
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
pub struct NFTokenBurn<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The NFToken to be removed by this transaction.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    /// The owner of the NFToken to burn. Only used if that owner is
    /// different than the account sending this transaction. The
    /// issuer or authorized minter can use this field to burn NFTs
    /// that have the lsfBurnable flag enabled.
    pub owner: Option<Cow<'a, str>>,
}

impl<'a> Model for NFTokenBurn<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for NFTokenBurn<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for NFTokenBurn<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> NFTokenBurn<'a> {
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
        nftoken_id: Cow<'a, str>,
        owner: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::NFTokenBurn,
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
            nftoken_id,
            owner,
        }
    }

    /// Set owner
    pub fn with_owner(mut self, owner: Cow<'a, str>) -> Self {
        self.owner = Some(owner);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2".into(),
                transaction_type: TransactionType::NFTokenBurn,
                fee: Some("10".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            owner: Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into()),
        };

        let default_json_str = r#"{"Account":"rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2","TransactionType":"NFTokenBurn","Fee":"10","Flags":0,"SigningPubKey":"","NFTokenID":"000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65","Owner":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenBurn = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let nftoken_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2".into(),
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        }
        .with_owner("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into())
        .with_fee("10".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("burning NFT".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            nftoken_burn.nftoken_id,
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
        );
        assert_eq!(
            nftoken_burn.owner.as_ref().unwrap(),
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"
        );
        assert_eq!(nftoken_burn.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(nftoken_burn.common_fields.sequence, Some(123));
        assert_eq!(
            nftoken_burn.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(nftoken_burn.common_fields.source_tag, Some(12345));
        assert_eq!(nftoken_burn.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let nftoken_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2".into(),
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        };

        assert_eq!(
            nftoken_burn.common_fields.account,
            "rNCFjv8Ek5oDrNiMJ3pw6eLLFtMjZLJnf2"
        );
        assert_eq!(
            nftoken_burn.common_fields.transaction_type,
            TransactionType::NFTokenBurn
        );
        assert_eq!(
            nftoken_burn.nftoken_id,
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
        );
        assert!(nftoken_burn.owner.is_none());
        assert!(nftoken_burn.common_fields.fee.is_none());
        assert!(nftoken_burn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_self_burn() {
        let self_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rTokenOwner123".into(),
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(456);

        assert!(self_burn.owner.is_none()); // Burning own NFT
        assert_eq!(self_burn.common_fields.account, "rTokenOwner123");
        assert_eq!(self_burn.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(self_burn.common_fields.sequence, Some(456));
    }

    #[test]
    fn test_authorized_burn() {
        let authorized_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rAuthorizedBurner456".into(), // Issuer or authorized account
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        }
        .with_owner("rActualOwner789".into()) // The actual owner of the NFT
        .with_fee("15".into())
        .with_sequence(789)
        .with_memo(Memo {
            memo_data: Some("authorized burn".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(authorized_burn.owner.as_ref().unwrap(), "rActualOwner789");
        assert_eq!(
            authorized_burn.common_fields.account,
            "rAuthorizedBurner456"
        );
        assert_eq!(authorized_burn.common_fields.fee.as_ref().unwrap().0, "15");
        assert_eq!(authorized_burn.common_fields.sequence, Some(789));
        assert_eq!(
            authorized_burn.common_fields.memos.as_ref().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rTicketUser111".into(),
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        }
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(ticket_burn.common_fields.ticket_sequence, Some(12345));
        assert_eq!(
            ticket_burn.nftoken_id,
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
        );
        // When using tickets, sequence should be None or 0
        assert!(ticket_burn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_burn = NFTokenBurn {
            common_fields: CommonFields {
                account: "rMemoUser222".into(),
                transaction_type: TransactionType::NFTokenBurn,
                ..Default::default()
            },
            nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".into(),
            ..Default::default()
        }
        .with_memo(Memo {
            memo_data: Some("reason 1".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("reason 2".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("12".into())
        .with_sequence(111);

        assert_eq!(
            multi_memo_burn.common_fields.memos.as_ref().unwrap().len(),
            2
        );
        assert_eq!(multi_memo_burn.common_fields.sequence, Some(111));
    }
}
