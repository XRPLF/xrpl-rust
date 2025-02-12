use alloc::borrow::Cow;
use alloc::vec::Vec;
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

use super::{CommonFields, FlagCollection};

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
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NFTokenMintFlag>,
    // The custom fields for the NFTokenMint model.
    //
    // See NFTokenMint fields:
    // `<https://xrpl.org/nftokenmint.html#nftokenmint-fields>`
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
    /// hexadecimal. You can use the xrpl.convertStringToHex  utility to convert a URI to
    /// its hexadecimal equivalent. This is intended to be a URI that points to the data or
    /// metadata associated with the NFT. The contents could decode to an HTTP or HTTPS URL,
    /// an IPFS URI, a magnet link, immediate data encoded as an RFC 2379 "data" URL , or
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
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::NFTokenMint,
                account_txn_id,
                fee,
                flags: flags.unwrap_or_default(),
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
            nftoken_taxon,
            issuer,
            transfer_fee,
            uri,
        }
    }
}

pub trait NFTokenMintError {
    fn _get_issuer_error(&self) -> XRPLModelResult<()>;
    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()>;
    fn _get_uri_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod test_nftoken_mint_error {

    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_issuer_error() {
        let nftoken_mint = NFTokenMint::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into()),
            None,
            None,
        );

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"issuer\"` is not allowed to be the same as the value of the field `\"account\"`"
        );
    }

    #[test]
    fn test_transfer_fee_error() {
        let nftoken_mint = NFTokenMint::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            None,
            Some(50001),
            None,
        );

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"transfer_fee\"` is defined above its maximum (max 50000, found 50001)"
        );
    }

    #[test]
    fn test_uri_error() {
        let nftoken_mint = NFTokenMint::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            None,
            None,
            Some("wss://xrplcluster.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
        );

        assert_eq!(
            nftoken_mint.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"uri\"` exceeds its maximum length of characters (max 512, found 513)"
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = NFTokenMint::new(
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
            None,
            Some("10".into()),
            Some(vec![NFTokenMintFlag::TfTransferable].into()),
            None,
            Some(vec![Memo::new(Some("72656E74".to_string()), None, Some("687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963".to_string()))]),
            None,
            None,
            None,
            None,
            0,
            None,
            Some(314),
            Some("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469".into()),
        );
        let default_json_str = r#"{"Account":"rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B","TransactionType":"NFTokenMint","Fee":"10","Flags":8,"Memos":[{"Memo":{"MemoData":"72656E74","MemoFormat":null,"MemoType":"687474703A2F2F6578616D706C652E636F6D2F6D656D6F2F67656E65726963"}}],"NFTokenTaxon":0,"TransferFee":314,"URI":"697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenMint = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
