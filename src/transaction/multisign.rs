// `multisign` now lives in `crate::signing`. Re-exported here for backward
// compatibility.
pub use crate::signing::multisign;

#[cfg(test)]
mod test {
    use alloc::borrow::Cow;

    use crate::asynch::transaction::sign;
    use crate::models::transactions::account_set::AccountSet;
    use crate::models::transactions::Transaction;
    use crate::signing::multisign;
    use crate::wallet::Wallet;

    #[tokio::test]
    async fn test_multisign() {
        let wallet = Wallet::new("sEdSkooMk31MeTjbHVE7vLvgCpEMAdB", 0).unwrap();
        let first_signer = Wallet::new("sEdTLQkHAWpdS7FDk7EvuS7Mz8aSMRh", 0).unwrap();
        let second_signer = Wallet::new("sEd7DXaHkGQD8mz8xcRLDxfMLqCurif", 0).unwrap();
        let mut account_set_txn = AccountSet::new(
            Cow::from(wallet.classic_address.clone()),
            None,
            Some("40".into()),
            None,
            Some(4814775),
            None,
            Some(4814738),
            None,
            None,
            None,
            None,
            Some("6578616d706c652e636f6d".into()), // "example.com"
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let mut tx_1 = account_set_txn.clone();
        sign(&mut tx_1, &first_signer, true).unwrap();
        let tx_1_expected_signature = "E3BEF86AEFC61E5ED66C95D0C5CE699721A8DAF86B6ED0D1CBAC86C2C03D96A098767B4F163FADBD937A99AC40BD6CED16B2CA98B198C2343D4BA31ECE57530C";
        assert_eq!(
            tx_1.get_common_fields().signers.as_ref().unwrap()[0]
                .txn_signature
                .as_str(),
            tx_1_expected_signature
        );
        let mut tx_2 = account_set_txn.clone();
        sign(&mut tx_2, &second_signer, true).unwrap();
        let tx_2_expected_signature = "DB64FC69F34A4881F6087226681E7BDDB212027B3FAFB617E598DCA5BBC8FA1A15A6E37A760B534BA554FBCD8D4A9FDEC8DFED206E3EBC393B875F59C765D304";
        assert_eq!(
            tx_2.get_common_fields().signers.as_ref().unwrap()[0]
                .txn_signature
                .as_str(),
            tx_2_expected_signature
        );
        let tx_list = [tx_1.clone(), tx_2.clone()].to_vec();
        multisign(&mut account_set_txn, &tx_list).unwrap();
        assert!(account_set_txn.get_common_fields().is_signed());
        assert_eq!(
            account_set_txn
                .get_common_fields()
                .signers
                .as_ref()
                .unwrap()
                .len(),
            2
        );
    }
}
