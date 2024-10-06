use crate::models::ledger::objects::LedgerEntryType;
use crate::models::FlagCollection;
use crate::models::NoFlags;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `Escrow` object type represents a held payment of XRP waiting to be executed or canceled.
/// An `EscrowCreate` transaction creates an `Escrow` object in the ledger. A successful `EscrowFinish`
/// or `EscrowCancel` transaction deletes the object. If the `Escrow` object has a crypto-condition,
/// the payment can only succeed if an `EscrowFinish` transaction provides the corresponding
/// fulfillment that satisfies the condition.
/// (The only supported crypto-condition type is PREIMAGE-SHA-256.) If the `Escrow` object has a
/// `FinishAfter` time, the held payment can only execute after that time.
///
/// An `Escrow` object is associated with two addresses:
/// - The owner, who provides the XRP when creating the `Escrow` object. If the held payment is
/// canceled, the XRP returns to the owner.
/// - The destination, where the XRP is paid when the held payment succeeds. The destination can
/// be the same as the owner.
///
/// `<https://xrpl.org/escrow-object.html#escrow>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Escrow<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the Escrow model.
    //
    // See Escrow fields:
    // `<https://xrpl.org/escrow-object.html#escrow-fields>`
    /// The address of the owner (sender) of this held payment. This is the account that provided
    /// the XRP, and gets it back if the held payment is canceled.
    pub account: Cow<'a, str>,
    /// The amount of XRP, in drops, to be delivered by the held payment.
    pub amount: Amount<'a>,
    /// The destination address where the XRP is paid if the held payment is successful.
    pub destination: Cow<'a, str>,
    /// A hint indicating which page of the owner directory links to this object, in case the
    /// directory consists of multiple pages. Note: The object does not contain a direct link
    /// to the owner directory containing it, since that value can be derived from the Account.
    pub owner_node: Cow<'a, str>,
    #[serde(rename = "PreviousTxnID")]
    /// The identifying hash of the transaction that most recently modified this object.
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The held payment can be canceled if and only if this field is present and the time it
    /// specifies has passed. Specifically, this is specified as seconds since the Ripple Epoch
    /// and it "has passed" if it's earlier than the close time of the previous validated ledger.
    pub cancel_after: Option<u32>,
    /// A PREIMAGE-SHA-256 crypto-condition, as hexadecimal. If present, the `EscrowFinish`
    /// transaction must contain a fulfillment that satisfies this condition.
    pub condition: Option<Cow<'a, str>>,
    /// A hint indicating which page of the destination's owner directory links to this object,
    /// in case the directory consists of multiple pages. Omitted on escrows created before
    /// enabling the fix1523 amendment.
    pub destination_node: Option<Cow<'a, str>>,
    /// An arbitrary tag to further specify the destination for this held payment, such as a
    /// hosted recipient at the destination address.
    pub destination_tag: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, after which this held payment can be finished.
    /// Any `EscrowFinish` transaction before this time fails.
    pub finish_after: Option<u32>,
    /// An arbitrary tag to further specify the source for this held payment, such as a hosted
    /// recipient at the owner's address.
    pub source_tag: Option<u32>,
}

impl<'a> Model for Escrow<'a> {}

impl<'a> LedgerObject<NoFlags> for Escrow<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Escrow<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        amount: Amount<'a>,
        destination: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        cancel_after: Option<u32>,
        condition: Option<Cow<'a, str>>,
        destination_node: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
        finish_after: Option<u32>,
        source_tag: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Escrow,
                index,
                ledger_index,
            },
            account,
            amount,
            destination,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            cancel_after,
            condition,
            destination_node,
            destination_tag,
            finish_after,
            source_tag,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::borrow::Cow;

    #[test]
    fn test_serialize() {
        let escrow = Escrow::new(
            Some(Cow::from(
                "DC5F3851D8A1AB622F957761E5963BC5BD439D5C24AC6AD7AC4523F0640244AC",
            )),
            None,
            Cow::from("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"),
            Amount::XRPAmount("10000".into()),
            Cow::from("ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"),
            Cow::from("0000000000000000"),
            Cow::from("C44F2EB84196B9AD820313DBEBA6316A15C9A2D35787579ED172B87A30131DA7"),
            28991004,
            Some(545440232),
            Some(Cow::from(
                "A0258020A82A88B2DF843A54F58772E4A3861866ECDB4157645DD9AE528C1D3AEEDABAB6810120",
            )),
            Some(Cow::from("0000000000000000")),
            Some(23480),
            Some(545354132),
            Some(11747),
        );
        let serialized = serde_json::to_string(&escrow).unwrap();

        let deserialized: Escrow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(escrow, deserialized);
    }
}
