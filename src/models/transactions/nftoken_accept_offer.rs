use alloc::borrow::Cow;
use alloc::vec::Vec;
use bigdecimal::{BigDecimal, Zero};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags, XRPLModelException, XRPLModelResult};

use super::CommonFields;

/// Accept offers to buy or sell an NFToken.
///
/// See NFTokenAcceptOffer:
/// `<https://xrpl.org/nftokenacceptoffer.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenAcceptOffer<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the NFTokenAcceptOffer model.
    //
    // See NFTokenAcceptOffer fields:
    // `<https://xrpl.org/nftokenacceptoffer.html#nftokenacceptoffer-fields>`
    /// Identifies the NFTokenOffer that offers to sell the NFToken.
    #[serde(rename = "NFTokenSellOffer")]
    pub nftoken_sell_offer: Option<Cow<'a, str>>,
    /// Identifies the NFTokenOffer that offers to buy the NFToken.
    #[serde(rename = "NFTokenBuyOffer")]
    pub nftoken_buy_offer: Option<Cow<'a, str>>,
    #[serde(rename = "NFTokenBrokerFee")]
    /// This field is only valid in brokered mode, and specifies the
    /// amount that the broker keeps as part of their fee for bringing
    /// the two offers together; the remaining amount is sent to the
    /// seller of the NFToken being bought. If specified, the fee must
    /// be such that, before applying the transfer fee, the amount that
    /// the seller would receive is at least as much as the amount
    /// indicated in the sell offer.
    pub nftoken_broker_fee: Option<Amount<'a>>,
}

impl<'a: 'static> Model for NFTokenAcceptOffer<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_brokered_mode_error()?;
        self._get_nftoken_broker_fee_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for NFTokenAcceptOffer<'a> {
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

impl<'a> NFTokenAcceptOfferError for NFTokenAcceptOffer<'a> {
    fn _get_brokered_mode_error(&self) -> XRPLModelResult<()> {
        if self.nftoken_broker_fee.is_some()
            && self.nftoken_sell_offer.is_none()
            && self.nftoken_buy_offer.is_none()
        {
            Err(XRPLModelException::ExpectedOneOf(&[
                "nftoken_sell_offer",
                "nftoken_buy_offer",
            ]))
        } else {
            Ok(())
        }
    }
    fn _get_nftoken_broker_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(nftoken_broker_fee) = &self.nftoken_broker_fee {
            let nftoken_broker_fee_decimal: BigDecimal = nftoken_broker_fee.clone().try_into()?;
            if nftoken_broker_fee_decimal.is_zero() {
                Err(XRPLModelException::ValueZero("nftoken_broker_fee".into()))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenAcceptOffer<'a> {
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
        nftoken_sell_offer: Option<Cow<'a, str>>,
        nftoken_buy_offer: Option<Cow<'a, str>>,
        nftoken_broker_fee: Option<Amount<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::NFTokenAcceptOffer,
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
            nftoken_sell_offer,
            nftoken_buy_offer,
            nftoken_broker_fee,
        }
    }
}

pub trait NFTokenAcceptOfferError {
    fn _get_brokered_mode_error(&self) -> XRPLModelResult<()>;
    fn _get_nftoken_broker_fee_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod test_nftoken_accept_offer_error {

    use alloc::string::ToString;

    use crate::models::{
        amount::{Amount, XRPAmount},
        Model,
    };

    use super::*;

    #[test]
    fn test_brokered_mode_error() {
        let nftoken_accept_offer = NFTokenAcceptOffer::new(
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
            None,
            Some(Amount::XRPAmount(XRPAmount::from("100"))),
        );

        assert_eq!(
            nftoken_accept_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "Expected one of: nftoken_sell_offer, nftoken_buy_offer"
        );
    }

    #[test]
    fn test_broker_fee_error() {
        let nftoken_accept_offer = NFTokenAcceptOffer::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("".into()),
            None,
            Some(Amount::XRPAmount(XRPAmount::from("0"))),
        );

        assert_eq!(
            nftoken_accept_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The value of the field `\"nftoken_broker_fee\"` is not allowed to be zero"
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
        let default_txn = NFTokenAcceptOffer::new(
            "r9spUPhPBfB6kQeF6vPhwmtFwRhBh2JUCG".into(),
            None,
            Some("12".into()),
            Some(75447550),
            Some(vec![Memo::new(
                Some(
                    "61356534373538372D633134322D346663382D616466362D393666383562356435386437"
                        .to_string(),
                ),
                None,
                None,
            )]),
            Some(68549302),
            None,
            None,
            None,
            Some("68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77".into()),
            None,
            None,
        );
        let default_json_str = r#"{"Account":"r9spUPhPBfB6kQeF6vPhwmtFwRhBh2JUCG","TransactionType":"NFTokenAcceptOffer","Fee":"12","Flags":0,"LastLedgerSequence":75447550,"Memos":[{"Memo":{"MemoData":"61356534373538372D633134322D346663382D616466362D393666383562356435386437","MemoFormat":null,"MemoType":null}}],"Sequence":68549302,"NFTokenSellOffer":"68CD1F6F906494EA08C9CB5CAFA64DFA90D4E834B7151899B73231DE5A0C3B77"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenAcceptOffer = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
