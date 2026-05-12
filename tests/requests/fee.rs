// Scenarios:
//   - base: send a fee request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{requests::fee::Fee as FeeRequest, results::fee::Fee as FeeResult};

#[tokio::test]
async fn test_fee_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(FeeRequest::new(None).into())
            .await
            .expect("fee request failed");

        let result: FeeResult = response.try_into().expect("failed to parse fee result");

        // Verify expected fields exist and have correct types
        assert!(!result.current_ledger_size.is_empty());
        assert!(!result.current_queue_size.is_empty());
        assert!(!result.expected_ledger_size.is_empty());
        assert!(result.ledger_current_index > 0);

        // Verify drops fields
        assert!(!result.drops.base_fee.0.is_empty());
        assert!(!result.drops.median_fee.0.is_empty());
        assert!(!result.drops.minimum_fee.0.is_empty());
        assert!(!result.drops.open_ledger_fee.0.is_empty());

        // Verify levels fields
        assert!(!result.levels.median_level.is_empty());
        assert!(!result.levels.minimum_level.is_empty());
        assert!(!result.levels.open_ledger_level.is_empty());
        assert!(!result.levels.reference_level.is_empty());
    })
    .await;
}
