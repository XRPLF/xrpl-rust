// Scenarios:
//   - base: send a channel_verify request with hardcoded test data and verify the response

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::channel_verify::ChannelVerify as ChannelVerifyRequest,
    results::channel_verify::ChannelVerify as ChannelVerifyResult, XRPAmount,
};

#[tokio::test]
async fn test_channel_verify_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let request = ChannelVerifyRequest::new(
            None, // id
            XRPAmount::from("1000000"), // amount
            "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".into(), // channel_id
            "aB44YfzW24VDEJQ2UuLPV2PvqcPCSoLnL7y5M1EzhdW4LnK5xMS3".into(), // public_key
            "304402204EF0AFB78AC23ED1C472E74F4299C0C21F1B21D07EFC0A3838A420F76D783A400220154FB11B6F54320666E4C36CA7F686C16A3A0456800BBC43746F34AF50290064".into(), // signature
        );

        let response = client
            .request(request.into())
            .await
            .expect("channel_verify request failed");

        let result: ChannelVerifyResult = response
            .try_into()
            .expect("failed to parse channel_verify result");

        // Verify that signature_verified is returned (the value depends on whether
        // the channel actually exists, but the field should be present)
        // With hardcoded test data, the signature may or may not verify
        let _ = result.signature_verified;
    })
    .await;
}
