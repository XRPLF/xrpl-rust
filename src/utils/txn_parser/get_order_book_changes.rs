use alloc::vec::Vec;

use crate::{
    models::transactions::metadata::TransactionMetadata, utils::exceptions::XRPLUtilsResult,
};

use super::utils::{order_book_parser::compute_order_book_changes, AccountOfferChanges};

pub fn get_order_book_changes<'a: 'b, 'b>(
    meta: &'a TransactionMetadata<'a>,
) -> XRPLUtilsResult<Vec<AccountOfferChanges<'b>>> {
    compute_order_book_changes(meta)
}

#[cfg(test)]
mod test {
    use core::cell::LazyCell;

    use serde_json::Value;

    use super::*;

    use crate::models::transactions::metadata::TransactionMetadata;

    #[test]
    fn test_get_order_book_changes() {
        let txn: LazyCell<TransactionMetadata> = LazyCell::new(|| {
            let txn_value: Value =
                serde_json::from_str(include_str!("./test_data/offer_created.json")).unwrap();
            let txn_meta = txn_value["meta"].clone();
            let txn_meta: TransactionMetadata = serde_json::from_value(txn_meta).unwrap();

            txn_meta
        });
        assert!(get_order_book_changes(&txn).is_ok());
    }
}
