use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::{
    constants::{MAX_TRANSFER_FEE, MAX_URI_LENGTH},
    models::{
        transactions::{Memo, Signer, Transaction, TransactionType},
        Model, XRPLModelException, XRPLModelResult,
    },
};

use crate::models::amount::XRPAmount;

use super::{CommonFields, CommonTransactionBuilder, FlagCollection};

/// Transactions of the NFTokenMint type support additional values
/// in the Flags field. This enum represents those options.
///
/// See NFTokenMint flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokenmint>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
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
    /// Allows the issuer (or an entity authorized by the issuer) to
    /// destroy the minted NFToken even if the NFToken is owned by another account.
    TfTrustLine = 0x00000004,
    /// The minted NFToken can be transferred to others. If this flag is not
    /// enabled, the token can still be transferred from or to the issuer.
    TfTransferable = 0x00000008,
}

impl TryFrom<u32> for NFTokenMintFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000001 => Ok(NFTokenMintFlag::TfBurnable),
            0x00000002 => Ok(NFTokenMintFlag::TfOnlyXRP),
            0x00000004 => Ok(NFTokenMintFlag::TfTrustLine),
            0x00000008 => Ok(NFTokenMintFlag::TfTransferable),
            _ => Err(()),
        }
    }
}

impl NFTokenMintFlag {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & 0x00000001 != 0 {
            flags.push(NFTokenMintFlag::TfBurnable);
        }
        if bits & 0x00000002 != 0 {
            flags.push(NFTokenMintFlag::TfOnlyXRP);
        }
        if bits & 0x00000004 != 0 {
            flags.push(NFTokenMintFlag::TfTrustLine);
        }
        if bits & 0x00000008 != 0 {
            flags.push(NFTokenMintFlag::TfTransferable);
        }
        flags
    }
}

/// The NFTokenMint transaction creates a non-fungible token and adds it to
/// the relevant NFTokenPage object of the NFTokenMinter as an NFToken object.
///
/// See NFTokenMint:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokenmint>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMint<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NFTokenMintFlag>,
    /// An arbitrary taxon, or shared identifier, for a series or collection of related NFTs.
    /// To mint a series of NFTs, give them all the same taxon.
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    /// The issuer of the token, if the sender of the account is issuing it on behalf of
    /// another account. This field must be omitted if the account sending the transaction
    /// is the issuer of the NFToken. If provided, the issuer's AccountRoot object must have
    /// the NFTokenMinter field set to the sender of this transaction (this transaction's
    /// Account field).
    pub issuer: Option<Cow<'a, str>>,
    /// The value specifies the fee charged by the issuer for secondary sales of the NFToken,
    /// if such sales are allowed. Valid values for this field are between 0 and 50000
    /// inclusive, allowing transfer rates of between 0.00% and 50.00% in increments of
    /// 0.001. If this field is provided, the transaction MUST have the tfTransferable
    /// flag enabled.
    pub transfer_fee: Option<u32>,
    /// Up to 256 bytes of arbitrary data. In JSON, this should be encoded as a string of
    /// hexadecimal. You can use the xrpl.convertStringToHex utility to convert a URI to
    /// its hexadecimal equivalent. This is intended to be a URI that points to the data or
    /// metadata associated with the NFT. The contents could decode to an HTTP or HTTPS URL,
    /// an IPFS URI, a magnet link, immediate data encoded as an RFC 2379 "data" URL, or
    /// even an issuer-specific encoding. The URI is NOT checked for validity.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
}

impl<'a> Model for NFTokenMint<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_issuer_error()?;
        self._get_transfer_fee_error()?;
        self._get_uri_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NFTokenMintFlag> for NFTokenMint<'a> {
    fn has_flag(&self, flag: &NFTokenMintFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NFTokenMintFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NFTokenMintFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NFTokenMintFlag> for NFTokenMint<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NFTokenMintFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> NFTokenMintError for NFTokenMint<'a> {
    fn _get_issuer_error(&self) -> XRPLModelResult<()> {
        if let Some(issuer) = &self.issuer {
            if issuer == &self.common_fields.account {
                Err(XRPLModelException::ValueEqualsValue {
                    field1: "issuer".into(),
                    field2: "account".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(transfer_fee) = self.transfer_fee {
            if transfer_fee > MAX_TRANSFER_FEE {
                Err(XRPLModelException::ValueTooHigh {
                    field: "transfer_fee".into(),
                    max: MAX_TRANSFER_FEE,
                    found: transfer_fee,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_uri_error(&self) -> XRPLModelResult<()> {
        if let Some(uri) = &self.uri {
            if uri.len() > MAX_URI_LENGTH {
                Err(XRPLModelException::ValueTooLong {
                    field: "uri".into(),
                    max: MAX_URI_LENGTH,
                    found: uri.len(),
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
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<NFTokenMintFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        nftoken_taxon: u32,
        issuer: Option<Cow<'a, str>>,
        transfer_fee: Option<u32>,
        uri: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::NFTokenMint,
                account_txn_id,
                fee,
                Some(flags.unwrap_or_default()),
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
            nftoken_taxon,
            issuer,
            transfer_fee,
            uri,
        }
    }

    /// Set issuer
    pub fn with_issuer(mut self, issuer: Cow<'a, str>) -> Self {
        self.issuer = Some(issuer);
        self
    }

    /// Set transfer fee
    pub fn with_transfer_fee(mut self, transfer_fee: u32) -> Self {
        self.transfer_fee = Some(transfer_fee);
        self
    }

    /// Set URI
    pub fn with_uri(mut self, uri: Cow<'a, str>) -> Self {
        self.uri = Some(uri);
        self
    }

    /// Add flag
    pub fn with_flag(mut self, flag: NFTokenMintFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    /// Set multiple flags
    pub fn with_flags(mut self, flags: Vec<NFTokenMintFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }
}

pub trait NFTokenMintError {
    fn _get_issuer_error(&self) -> XRPLModelResult<()>;
    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()>;
    fn _get_uri_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use core::convert::TryFrom;

    use crate::models::Model;
    use super::*;

    #[test]
    fn test_issuer_error() {
        let nftoken_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 0,
            issuer: Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"issuer\"` is not allowed to be the same as the value of the field `\"account\"`"
        );
    }

    #[test]
    fn test_transfer_fee_error() {
        let nftoken_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 0,
            transfer_fee: Some(50001),
            ..Default::default()
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"transfer_fee\"` is defined above its maximum (max 50000, found 50001)"
        );
    }

    #[test]
    fn test_uri_error() {
        let nftoken_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 0,
            uri: Some("wss://xrplcluster.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"uri\"` exceeds its maximum length of characters (max 512, found 513)"
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = NFTokenMint {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::NFTokenMint,
                fee: Some("10".into()),
                flags: vec![NFTokenMintFlag::TfTransferable].into(),
                memos: Some(vec![Memo::new(
                    Some("72656E74".to_string()), 
                    None, 
                    Some("687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963".to_string())
                )]),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_taxon: 0,
            transfer_fee: Some(314),
            uri: Some("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469".into()),
            ..Default::default()
        };
        
        let default_json_str = r#"{"Account":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B","TransactionType":"NFTokenMint","Fee":"10","Flags":8,"Memos":[{"Memo":{"MemoData":"72656E74","MemoFormat":null,"MemoType":"687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963"}}],"SigningPubKey":"","NFTokenTaxon":0,"TransferFee":314,"URI":"697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"}"#;
        
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenMint = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let nftoken_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 12345,
            ..Default::default()
        }
        .with_issuer("rLsn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into())
        .with_transfer_fee(314)
        .with_uri("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469".into())
        .with_flags(vec![NFTokenMintFlag::TfTransferable, NFTokenMintFlag::TfBurnable])
        .with_fee("10".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo::new(
            Some("creating NFT".into()),
            None,
            Some("text".into())
        ));

        assert_eq!(nftoken_mint.nftoken_taxon, 12345);
        assert_eq!(nftoken_mint.issuer.as_ref().unwrap(), "rLsn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK");
        assert_eq!(nftoken_mint.transfer_fee, Some(314));
        assert!(nftoken_mint.uri.is_some());
        assert!(nftoken_mint.has_flag(&NFTokenMintFlag::TfTransferable));
        assert!(nftoken_mint.has_flag(&NFTokenMintFlag::TfBurnable));
        assert_eq!(nftoken_mint.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(nftoken_mint.common_fields.sequence, Some(123));
        assert_eq!(nftoken_mint.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(nftoken_mint.common_fields.source_tag, Some(12345));
        assert_eq!(nftoken_mint.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let nftoken_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 0,
            ..Default::default()
        };

        assert_eq!(nftoken_mint.common_fields.account, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
        assert_eq!(nftoken_mint.common_fields.transaction_type, TransactionType::NFTokenMint);
        assert_eq!(nftoken_mint.nftoken_taxon, 0);
        assert!(nftoken_mint.issuer.is_none());
        assert!(nftoken_mint.transfer_fee.is_none());
        assert!(nftoken_mint.uri.is_none());
    }

    #[test]
    fn test_collection_minting() {
        let collection_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 99999, // Collection identifier
            ..Default::default()
        }
        .with_flags(vec![NFTokenMintFlag::TfTransferable, NFTokenMintFlag::TfOnlyXRP])
        .with_transfer_fee(500) // 0.5%
        .with_uri("ipfs://collection-metadata-hash".into())
        .with_fee("15".into())
        .with_sequence(456);

        assert_eq!(collection_mint.nftoken_taxon, 99999);
        assert!(collection_mint.has_flag(&NFTokenMintFlag::TfTransferable));
        assert!(collection_mint.has_flag(&NFTokenMintFlag::TfOnlyXRP));
        assert_eq!(collection_mint.transfer_fee, Some(500));
        assert!(collection_mint.uri.is_some());
        assert!(collection_mint.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_mint = NFTokenMint {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::NFTokenMint,
                ..Default::default()
            },
            nftoken_taxon: 888,
            ..Default::default()
        }
        .with_ticket_sequence(789)
        .with_flag(NFTokenMintFlag::TfBurnable)
        .with_fee("12".into());

        assert_eq!(ticket_mint.common_fields.ticket_sequence, Some(789));
        assert_eq!(ticket_mint.nftoken_taxon, 888);
        assert!(ticket_mint.has_flag(&NFTokenMintFlag::TfBurnable));
        // When using tickets, sequence should be None or 0
        assert!(ticket_mint.common_fields.sequence.is_none());
    }

    #[test]
    fn test_try_from_u32() {
        let cases = [
            (0x00000001, Ok(NFTokenMintFlag::TfBurnable)),
            (0x00000002, Ok(NFTokenMintFlag::TfOnlyXRP)),
            (0x00000004, Ok(NFTokenMintFlag::TfTrustLine)),
            (0x00000008, Ok(NFTokenMintFlag::TfTransferable)),
            (0x00000010, Err(())), // invalid flag
            (0x00000009, Err(())), // not a single flag
            (0x00000000, Err(())), // zero is not a valid single flag
        ];

        for (input, expected) in cases {
            assert_eq!(
                NFTokenMintFlag::try_from(input),
                expected,
                "try_from({:#X}) failed",
                input
            );
        }
    }

    #[test]
    fn test_from_bits() {
        use NFTokenMintFlag::*;
        let cases = [
            (0x00000001, vec![TfBurnable]),
            (0x00000002, vec![TfOnlyXRP]),
            (0x00000004, vec![TfTrustLine]),
            (0x00000008, vec![TfTransferable]),
            (0x00000009, vec![TfBurnable, TfTransferable]),
            (0x0000000B, vec![TfBurnable, TfOnlyXRP, TfTransferable]),
            (
                0x0000000F,
                vec![TfBurnable, TfOnlyXRP, TfTrustLine, TfTransferable],
            ),
            (0x00000000, vec![]),
            (0x00000003, vec![TfBurnable, TfOnlyXRP]),
            (0x00000005, vec![TfBurnable, TfTrustLine]),
            (0x0000000C, vec![TfTrustLine, TfTransferable]),
        ];

        for (input, ref expected) in cases {
            let mut actual = NFTokenMintFlag::from_bits(input);
            let mut expected_sorted = expected.clone();
            actual.sort_by_key(|f| *f as u32);
            expected_sorted.sort_by_key(|f| *f as u32);
            assert_eq!(actual, expected_sorted, "from_bits({:#X}) failed", input);
        }
    }
}
