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
    use alloc::{dbg, vec};

    use super::*;
    use crate::asynch::clients::XRPLAsyncClient;
    use crate::asynch::transaction::{autofill, sign};
    use crate::asynch::wallet::generate_faucet_wallet;
    use crate::clients::json_rpc::JsonRpcClient;
    use crate::models::requests::submit_multisigned::SubmitMultisigned;
    use crate::models::transactions::account_set::AccountSet;
    use crate::models::transactions::signer_list_set::SignerEntry;
    use crate::wallet::Wallet;

    #[tokio::test]
    async fn test_multisign() {
        let client =
            JsonRpcClient::connect("https://s.altnet.rippletest.net:51234".parse().unwrap());
        let wallet = Wallet::create(None).unwrap();
        let wallet = generate_faucet_wallet(&client, Some(wallet), None, None, None)
            .await
            .unwrap();
        let first_signer = Wallet::new("sEdTLQkHAWpdS7FDk7EvuS7Mz8aSMRh", 0).unwrap();
        let second_signer = Wallet::new("sEd7DXaHkGQD8mz8xcRLDxfMLqCurif", 0).unwrap();
        let signer_entries = vec![
            SignerEntry::new(first_signer.classic_address.clone(), 1),
            SignerEntry::new(second_signer.classic_address.clone(), 1),
        ];
        let mut account_set_txn = AccountSet::new(
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
        autofill(
            &mut account_set_txn,
            &client,
            Some(signer_entries.len().try_into().unwrap()),
        )
        .await
        .unwrap();
        let mut tx_1 = account_set_txn.clone();
        sign(&mut tx_1, &first_signer, true).unwrap();
        let mut tx_2 = account_set_txn.clone();
        sign(&mut tx_2, &second_signer, true).unwrap();
        let tx_list = [tx_1.clone(), tx_2.clone()].to_vec();
        dbg!(&account_set_txn, &tx_list);
        multisign(&mut account_set_txn, &tx_list).unwrap();
        dbg!(&account_set_txn);
        assert!(account_set_txn.get_common_fields().is_signed());
        let res = client
            .request(
                SubmitMultisigned::new(None, serde_json::to_value(&account_set_txn).unwrap(), None)
                    .into(),
            )
            .await;
        dbg!(&res);
    }
}
