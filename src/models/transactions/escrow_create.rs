use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags, XRPLModelException, XRPLModelResult};

/// Creates an Escrow, which requests XRP until the escrow process either finishes or is canceled.
///
/// See EscrowCreate:
/// `<https://xrpl.org/escrowcreate.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the EscrowCreate model.
    //
    // See EscrowCreate fields:
    // `<https://xrpl.org/escrowcreate.html>`
    /// Amount of XRP, in drops, to deduct from the sender's balance and escrow.
    /// Once escrowed, the XRP can either go to the Destination address
    /// (after the FinishAfter time) or returned to the sender (after the CancelAfter time).
    pub amount: XRPAmount<'a>,
    /// Address to receive escrowed XRP.
    pub destination: Cow<'a, str>,
    /// Arbitrary tag to further specify the destination for this escrowed
    /// payment, such as a hosted recipient at the destination address.
    pub destination_tag: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, when this
    /// escrow expires. This value is immutable; the funds can
    /// only be returned to the sender after this time.
    pub cancel_after: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, when the escrowed XRP
    /// can be released to the recipient. This value is immutable, and the
    /// funds can't be accessed until this time.
    pub finish_after: Option<u32>,
    /// Hex value representing a PREIMAGE-SHA-256 crypto-condition.
    /// The funds can only be delivered to the recipient if this
    /// condition is fulfilled. If the condition is not fulfilled
    /// before the expiration time specified in the CancelAfter
    /// field, the XRP can only revert to the sender.
    pub condition: Option<Cow<'a, str>>,
}

impl<'a: 'static> Model for EscrowCreate<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_finish_after_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for EscrowCreate<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> EscrowCreateError for EscrowCreate<'a> {
    fn _get_finish_after_error(&self) -> XRPLModelResult<()> {
        if let (Some(finish_after), Some(cancel_after)) = (self.finish_after, self.cancel_after) {
            if finish_after >= cancel_after {
                Err(XRPLModelException::ValueBelowValue {
                    field1: "cancel_after".into(),
                    field2: "finish_after".into(),
                    field1_val: cancel_after,
                    field2_val: finish_after,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> EscrowCreate<'a> {
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
        amount: XRPAmount<'a>,
        destination: Cow<'a, str>,
        cancel_after: Option<u32>,
        condition: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
        finish_after: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::EscrowCreate,
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
            destination,
            destination_tag,
            cancel_after,
            finish_after,
            condition,
        }
    }
}

pub trait EscrowCreateError {
    fn _get_finish_after_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod test_escrow_create_errors {
    use crate::models::Model;

    use crate::models::amount::XRPAmount;

    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_cancel_after_error() {
        let escrow_create = EscrowCreate::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("100000000"),
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            Some(13298498),
            None,
            None,
            Some(14359039),
        );

        assert_eq!(
            escrow_create.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"cancel_after\"` is not allowed to be below the value of the field `\"finish_after\"` (max 14359039, found 13298498)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = EscrowCreate::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(11747),
            None,
            XRPAmount::from("10000"),
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            Some(533257958),
            Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            Some(23480),
            Some(533171558),
        );
        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"EscrowCreate","Flags":0,"SourceTag":11747,"Amount":"10000","Destination":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","DestinationTag":23480,"CancelAfter":533257958,"FinishAfter":533171558,"Condition":"A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: EscrowCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
