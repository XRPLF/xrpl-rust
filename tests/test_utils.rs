#[cfg(feature = "utils")]
use xrpl::utils::{posix_to_ripple_time, ripple_time_to_posix};

#[test]
#[cfg(feature = "utils")]
fn it_converts_posix_to_ripple_time() {
    assert_eq!(posix_to_ripple_time(1660187459), Ok(713502659_i64));
}

#[test]
#[cfg(feature = "utils")]
fn it_converts_ripple_time_to_posix() {
    assert_eq!(ripple_time_to_posix(713502659), Ok(1660187459));
}

#[cfg(all(feature = "std", feature = "json-rpc", feature = "websocket"))]
mod exception_mapping {
    use std::error::Error;

    use xrpl::asynch::clients::{
        exceptions::{XRPLClientException, XRPLNetworkErrorKind},
        XRPLJsonRpcException, XRPLWebSocketException,
    };

    #[test]
    fn maps_websocket_network_error_kinds() {
        let cases = [
            (
                XRPLWebSocketException::Disconnected,
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLWebSocketException::ConnectionClosed,
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLWebSocketException::AlreadyClosed,
                Some(XRPLNetworkErrorKind::AlreadyClosed),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(std::io::Error::from(
                    std::io::ErrorKind::ConnectionRefused,
                )),
                Some(XRPLNetworkErrorKind::ConnectionRefused),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(std::io::Error::from(
                    std::io::ErrorKind::ConnectionReset,
                )),
                Some(XRPLNetworkErrorKind::ConnectionReset),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(std::io::Error::from(
                    std::io::ErrorKind::TimedOut,
                )),
                Some(XRPLNetworkErrorKind::TimedOut),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(std::io::Error::from(
                    std::io::ErrorKind::Other,
                )),
                Some(XRPLNetworkErrorKind::OtherIo),
            ),
            (
                XRPLWebSocketException::Tls("tls".into()),
                Some(XRPLNetworkErrorKind::Tls),
            ),
            (
                XRPLWebSocketException::Protocol("protocol".into()),
                Some(XRPLNetworkErrorKind::Protocol),
            ),
            (
                XRPLWebSocketException::InvalidMessage,
                Some(XRPLNetworkErrorKind::InvalidResponse),
            ),
            (
                XRPLWebSocketException::UnexpectedMessageType,
                Some(XRPLNetworkErrorKind::InvalidResponse),
            ),
            (XRPLWebSocketException::Capacity("capacity".into()), None),
        ];

        for (error, expected) in cases {
            assert_eq!(error.network_error_kind(), expected);
        }
    }

    #[test]
    fn maps_client_network_error_kinds() {
        let cases = [
            (
                XRPLClientException::XRPLWebSocketError(Box::new(
                    XRPLWebSocketException::ConnectionClosed,
                )),
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::RequestError(
                    "not transport".into(),
                )),
                None,
            ),
            (
                XRPLClientException::IoError(std::io::Error::from(
                    std::io::ErrorKind::ConnectionRefused,
                )),
                Some(XRPLNetworkErrorKind::ConnectionRefused),
            ),
            (
                XRPLClientException::IoError(std::io::Error::from(
                    std::io::ErrorKind::ConnectionReset,
                )),
                Some(XRPLNetworkErrorKind::ConnectionReset),
            ),
            (
                XRPLClientException::IoError(std::io::Error::from(std::io::ErrorKind::TimedOut)),
                Some(XRPLNetworkErrorKind::TimedOut),
            ),
            (
                XRPLClientException::IoError(std::io::Error::from(std::io::ErrorKind::Other)),
                Some(XRPLNetworkErrorKind::OtherIo),
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.network_error_kind(), expected);
        }
    }

    #[test]
    fn maps_json_rpc_network_error_kinds() {
        let request_error = XRPLJsonRpcException::RequestError("bad request".into());
        assert_eq!(request_error.network_error_kind(), None);

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let connect_error = runtime.block_on(async {
            reqwest::get("http://127.0.0.1:1")
                .await
                .expect_err("localhost port 1 should refuse connections")
        });
        let json_rpc_error = XRPLJsonRpcException::ReqwestError(connect_error);
        assert_eq!(
            json_rpc_error.network_error_kind(),
            Some(XRPLNetworkErrorKind::OtherIo)
        );
    }

    #[test]
    fn converts_tungstenite_errors_to_typed_websocket_errors() {
        let cases = [
            (
                tokio_tungstenite::tungstenite::Error::ConnectionClosed,
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                tokio_tungstenite::tungstenite::Error::AlreadyClosed,
                Some(XRPLNetworkErrorKind::AlreadyClosed),
            ),
            (
                tokio_tungstenite::tungstenite::Error::Io(std::io::Error::from(
                    std::io::ErrorKind::ConnectionRefused,
                )),
                Some(XRPLNetworkErrorKind::ConnectionRefused),
            ),
        ];

        for (error, expected) in cases {
            let error = XRPLWebSocketException::from(error);
            assert_eq!(error.network_error_kind(), expected);
        }

        let protocol_error = tokio_tungstenite::tungstenite::Error::Protocol(
            tokio_tungstenite::tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
        );
        let protocol_error = XRPLWebSocketException::from(protocol_error);
        assert_eq!(
            protocol_error.network_error_kind(),
            Some(XRPLNetworkErrorKind::Protocol)
        );
        assert!(protocol_error.source().is_none());
    }
}
