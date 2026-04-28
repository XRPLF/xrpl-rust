// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/accountSet.test.ts
//
// Scenarios:
//   - base: set domain field with hex-encoded value
//   - with_memo: attach a memo to the transaction
//   - clawback_flag_*: XLS-0039 Section 3.2 – AllowTrustLineClawback flag rules

use crate::common::{generate_funded_wallet, get_client, test_transaction, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::transactions::{
    account_set::{AccountSet, AccountSetFlag},
    Memo,
};

#[tokio::test]
async fn test_account_set_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = AccountSet::new(
            wallet.classic_address.clone().into(),
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
            Some("6578616d706c652e636f6d".into()), // hex("example.com")
            None,
            None,
            None,
            None,
            None,
            None,
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

#[tokio::test]
async fn test_account_set_with_memo() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = AccountSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            Some(vec![Memo::new(
                Some(hex::encode("Hello, XRPL!").into()),
                Some(hex::encode("text/plain").into()),
                Some(hex::encode("application/json").into()),
            )]),
            None,
            None,
            None,
            None,
            None,
            Some("6578616d706c652e636f6d".into()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// XLS-0039 Section 3.2 – AllowTrustLineClawback flag
// ---------------------------------------------------------------------------

/// baseline: a fresh account with an empty owner directory can
/// successfully enable AsfAllowTrustLineClawback.
#[tokio::test]
async fn test_clawback_flag_set_on_empty_account() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = AccountSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,                                            // clear_flag
            None,                                            // domain
            None,                                            // email_hash
            None,                                            // message_key
            Some(AccountSetFlag::AsfAllowTrustLineClawback), // set_flag
            None,                                            // transfer_rate
            None,                                            // tick_size
            None,                                            // nftoken_minter
        );

        test_transaction(&mut tx, &wallet).await;

        // Verify the flag is now set via account_info
        let client = get_client().await;
        let request = xrpl::models::requests::account_info::AccountInfo::new(
            None,
            wallet.classic_address.clone().into(),
            None,
            Some("current".into()),
            None,
            None,
            None,
        );
        let response = client.request(request.into()).await.unwrap();
        let account_info =
            xrpl::models::results::account_info::AccountInfoVersionMap::try_from(response).unwrap();
        let account_flags = match &account_info {
            xrpl::models::results::account_info::AccountInfoVersionMap::Default(i) => {
                &i.base.account_flags
            }
            xrpl::models::results::account_info::AccountInfoVersionMap::V1(i) => {
                &i.base.account_flags
            }
        };
        assert!(
            account_flags
                .as_ref()
                .expect("account_flags should be present")
                .allow_trust_line_clawback,
            "AllowTrustLineClawback should be true after AccountSet"
        );
    })
    .await;
}
