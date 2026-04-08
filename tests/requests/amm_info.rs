// Scenarios:
//   - base: set up an AMM pool and query its info using amm_info

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::amm_info::AMMInfo as AMMInfoRequest, results::amm_info::AMMInfo as AMMInfoResult,
    Currency, IssuedCurrency, XRP,
};

#[tokio::test]
async fn test_amm_info_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let pool = crate::common::amm::setup_amm_pool().await;

        let request = AMMInfoRequest::new(
            None, // id
            None, // amm_account
            Some(Currency::XRP(XRP::new())),
            Some(Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                pool.issuer_wallet.classic_address.clone().into(),
            ))),
        );

        let response = client
            .request(request.into())
            .await
            .expect("amm_info request failed");

        let result: AMMInfoResult = response
            .try_into()
            .expect("failed to parse amm_info result");

        // Verify the AMM description has valid data
        assert!(!result.amm.account.is_empty());
        assert_eq!(result.amm.trading_fee, 12);
    })
    .await;
}
