use crate::models::{transactions::metadata::NodeType, Amount};

use super::{nodes::NormalizedNode, Balance, OfferStatus};

const LSF_SELL: u32 = 0x00020000;

fn get_offer_status(node: &NormalizedNode<'_>) -> OfferStatus {
    match node.node_type {
        NodeType::CreatedNode => OfferStatus::Created,
        NodeType::ModifiedNode => OfferStatus::PartiallyFilled,
        NodeType::DeletedNode => {
            if let Some(_) = node.previous_fields {
                // a filled offer has previous fields
                OfferStatus::Filled
            } else {
                OfferStatus::Cancelled
            }
        }
    }
}

fn derive_currency_amount<'a: 'b, 'b>(currency_amount: &'a Amount) -> Balance<'b> {
    match currency_amount {
        Amount::XRPAmount(amount) => Balance {
            currency: "XRP".into(),
            value: amount.0.clone(),
            issuer: None,
        },
        Amount::IssuedCurrencyAmount(amount) => Balance {
            currency: amount.currency.clone(),
            value: amount.value.clone(),
            issuer: Some(amount.issuer.clone()),
        },
    }
}
