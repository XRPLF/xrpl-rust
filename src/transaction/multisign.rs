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
        let wallet = Wallet::new("sEdT7wHTCLzDG7ueaw4hroSTBvH7Mk5", 0).unwrap();
        let wallet1 = Wallet::create(None).unwrap();
        let wallet2 = Wallet::create(None).unwrap();
        let mut multi_signed_tx = AccountSet::new(
            Cow::from(wallet.classic_address.clone()),
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
            Some("6578616d706c652e636f6d".into()), // "example.com"
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let mut tx_1 = multi_signed_tx.clone();
        sign(&mut tx_1, &wallet1, true).unwrap();
        let mut tx_2 = multi_signed_tx.clone();
        sign(&mut tx_2, &wallet2, true).unwrap();
        let tx_list = [tx_1.clone(), tx_2.clone()].to_vec();

        multisign(&mut multi_signed_tx, &tx_list).unwrap();
        assert!(multi_signed_tx.get_common_fields().is_signed());
    }
}
