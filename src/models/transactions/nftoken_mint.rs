use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::{
    constants::{MAX_TRANSFER_FEE, MAX_URI_LENGTH},
    models::{
        model::Model,
        transactions::{Flag, Memo, Signer, Transaction, TransactionType},
    },
    Err,
};

use crate::_serde::txn_flags;
use crate::models::amount::XRPAmount;
use crate::models::transactions::XRPLNFTokenMintException;

/// Transactions of the NFTokenMint type support additional values
/// in the Flags field. This enum represents those options.
///
/// See NFTokenMint flags:
/// `<https://xrpl.org/nftokenmint.html#nftokenmint-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum NFTokenMintFlag {
    /// Allow the issuer (or an entity authorized by the issuer) to
    /// destroy the minted NFToken. (The NFToken's owner can always do so.)
    TfBurnable = 0x00000001,
    /// The minted NFToken can only be bought or sold for XRP.
    /// This can be desirable if the token has a transfer fee and the issuer
    /// does not want to receive fees in non-XRP currencies.
    TfOnlyXRP = 0x00000002,
    /// The minted NFToken can be transferred to others. If this flag is not
    /// enabled, the token can still be transferred from or to the issuer.
    TfTransferable = 0x00000008,
}

/// The NFTokenMint transaction creates a non-fungible token and adds it to
/// the relevant NFTokenPage object of the NFTokenMinter as an NFToken object.
///
/// See NFTokenMint:
/// `<https://xrpl.org/nftokenmint.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMint<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::nftoken_mint")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: &'a str,
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
    pub account_txn_id: Option<&'a str>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<&'a str>,
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
    pub txn_signature: Option<&'a str>,
    /// Set of bit-flags for this transaction.
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<NFTokenMintFlag>>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the NFTokenMint model.
    ///
    /// See NFTokenMint fields:
    /// `<https://xrpl.org/nftokenmint.html#nftokenmint-fields>`
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    pub issuer: Option<&'a str>,
    pub transfer_fee: Option<u32>,
    #[serde(rename = "URI")]
    pub uri: Option<&'a str>,
}

impl<'a> Default for NFTokenMint<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::NFTokenMint,
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
            nftoken_taxon: Default::default(),
            issuer: Default::default(),
            transfer_fee: Default::default(),
            uri: Default::default(),
        }
    }
}

impl<'a: 'static> Model for NFTokenMint<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_issuer_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => match self._get_transfer_fee_error() {
                Err(error) => Err!(error),
                Ok(_no_error) => match self._get_uri_error() {
                    Err(error) => Err!(error),
                    Ok(_no_error) => Ok(()),
                },
            },
        }
    }
}

impl<'a> Transaction for NFTokenMint<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        let mut flags = &Vec::new();

        if let Some(flag_set) = self.flags.as_ref() {
            flags = flag_set;
        }

        match flag {
            Flag::NFTokenMint(nftoken_mint_flag) => match nftoken_mint_flag {
                NFTokenMintFlag::TfBurnable => flags.contains(&NFTokenMintFlag::TfBurnable),
                NFTokenMintFlag::TfOnlyXRP => flags.contains(&NFTokenMintFlag::TfOnlyXRP),
                NFTokenMintFlag::TfTransferable => flags.contains(&NFTokenMintFlag::TfTransferable),
            },
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> NFTokenMintError for NFTokenMint<'a> {
    fn _get_issuer_error(&self) -> Result<(), XRPLNFTokenMintException> {
        if let Some(issuer) = self.issuer {
            if issuer == self.account {
                Err(XRPLNFTokenMintException::ValueEqualsValue {
                    field1: "issuer",
                    field2: "account",
                    resource: "",
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_transfer_fee_error(&self) -> Result<(), XRPLNFTokenMintException> {
        if let Some(transfer_fee) = self.transfer_fee {
            if transfer_fee > MAX_TRANSFER_FEE {
                Err(XRPLNFTokenMintException::ValueTooHigh {
                    field: "transfer_fee",
                    max: MAX_TRANSFER_FEE,
                    found: transfer_fee,
                    resource: "",
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_uri_error(&self) -> Result<(), XRPLNFTokenMintException> {
        if let Some(uri) = self.uri {
            if uri.len() > MAX_URI_LENGTH {
                Err(XRPLNFTokenMintException::ValueTooLong {
                    field: "uri",
                    max: MAX_URI_LENGTH,
                    found: uri.len(),
                    resource: "",
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenMint<'a> {
    pub fn new(
        account: &'a str,
        nftoken_taxon: u32,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        flags: Option<Vec<NFTokenMintFlag>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        issuer: Option<&'a str>,
        transfer_fee: Option<u32>,
        uri: Option<&'a str>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::NFTokenMint,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags,
            memos,
            signers,
            nftoken_taxon,
            issuer,
            transfer_fee,
            uri,
        }
    }
}

pub trait NFTokenMintError {
    fn _get_issuer_error(&self) -> Result<(), XRPLNFTokenMintException>;
    fn _get_transfer_fee_error(&self) -> Result<(), XRPLNFTokenMintException>;
    fn _get_uri_error(&self) -> Result<(), XRPLNFTokenMintException>;
}

#[cfg(test)]
mod test_nftoken_mint_error {

    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_issuer_error() {
        let nftoken_mint = NFTokenMint {
            transaction_type: TransactionType::NFTokenMint,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            nftoken_taxon: 0,
            issuer: Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb"),
            transfer_fee: None,
            uri: None,
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `issuer` is not allowed to be the same as the value of the field `account`. For more information see: "
        );
    }

    #[test]
    fn test_transfer_fee_error() {
        let nftoken_mint = NFTokenMint {
            transaction_type: TransactionType::NFTokenMint,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            nftoken_taxon: 0,
            issuer: None,
            transfer_fee: Some(50001),
            uri: None,
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The field `transfer_fee` exceeds its maximum value (max 50000, found 50001). For more information see: "
        );
    }

    #[test]
    fn test_uri_error() {
        let nftoken_mint = NFTokenMint {
            transaction_type: TransactionType::NFTokenMint,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            nftoken_taxon: 0,
            issuer: None,
            transfer_fee: None,
            uri: Some("wss://xrplcluster.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `uri` exceeds its maximum length of characters (max 512, found 513). For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = NFTokenMint::new(
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            0,
            Some("10".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![NFTokenMintFlag::TfTransferable]),
            Some(vec![Memo::new(Some("72656E74".to_string()), None, Some("687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963".to_string()))]),
            None,
            None,
            Some(314),
            Some("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"),
        );
        let default_json = r#"{"TransactionType":"NFTokenMint","Account":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B","Fee":"10","Flags":8,"Memos":[{"Memo":{"MemoData":"72656E74","MemoFormat":null,"MemoType":"687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963"}}],"NFTokenTaxon":0,"TransferFee":314,"URI":"697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = NFTokenMint::new(
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            0,
            Some("10".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![NFTokenMintFlag::TfTransferable]),
            Some(vec![Memo::new(Some("72656E74".to_string()), None, Some("687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963".to_string()))]),
            None,
            None,
            Some(314),
            Some("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"),
        );
        let default_json = r#"{"TransactionType":"NFTokenMint","Account":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B","TransferFee":314,"NFTokenTaxon":0,"Flags":8,"Fee":"10","URI":"697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469","Memos":[{"Memo":{"MemoType":"687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963","MemoFormat":null,"MemoData":"72656E74"}}]}"#;

        let txn_as_obj: NFTokenMint = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
