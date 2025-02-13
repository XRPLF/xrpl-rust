use core::fmt::Debug;

use alloc::vec::Vec;
use serde::Serialize;
use strum::IntoEnumIterator;

use crate::{
    asynch::exceptions::XRPLHelperResult, core::addresscodec::decode_classic_address,
    models::transactions::Transaction, transaction::exceptions::XRPLMultisignException,
};

pub fn multisign<'a, T, F>(transaction: &mut T, tx_list: &'a Vec<T>) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq + 'a,
    T: Transaction<'a, F>,
{
    let mut decoded_tx_signers = Vec::new();
    for tx in tx_list {
        let tx_signers = match tx.get_common_fields().signers.as_ref() {
            Some(signers) => signers,
            None => return Err(XRPLMultisignException::NoSigners.into()),
        };
        let tx_signer = match tx_signers.first() {
            Some(signer) => signer,
            None => return Err(XRPLMultisignException::NoSigners.into()),
        };
        decoded_tx_signers.push(tx_signer.clone());
    }
    decoded_tx_signers
        .sort_by_key(|signer| decode_classic_address(signer.account.as_ref()).unwrap());
    transaction.get_mut_common_fields().signers = Some(decoded_tx_signers);
    transaction.get_mut_common_fields().signing_pub_key = Some("".into());

    Ok(())
}

#[cfg(test)]
mod test {
    use alloc::borrow::Cow;

    use super::*;
    use crate::asynch::transaction::sign;
    use crate::models::transactions::account_set::AccountSet;
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
